use crate::config::AppConfig;
use crate::errors::{Result, SpiceError};
use crate::models::{AppReleaseAsset, AppUpdateState, AppUpdateStatusKind};
use chrono::Utc;
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::Read;

#[derive(Debug, Deserialize)]
struct GithubRelease {
    html_url: String,
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    content_type: Option<String>,
}

pub fn check_latest(config: &AppConfig, owner: &str, repo: &str) -> Result<AppUpdateState> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    let client = reqwest::blocking::Client::builder()
        .user_agent("SpiceManager app-update")
        .build()?;
    let response = client.get(url).send()?;
    if response.status().as_u16() == 404 {
        return Ok(AppUpdateState {
            status: AppUpdateStatusKind::NoReleaseFound,
            current_version,
            latest_version: None,
            release_url: None,
            selected_asset: None,
            downloaded_asset_path: None,
            last_checked_at: Some(Utc::now()),
            last_error: None,
        });
    }
    let release: GithubRelease = response.error_for_status()?.json()?;
    if release.draft || release.prerelease {
        return Ok(AppUpdateState {
            status: AppUpdateStatusKind::NoReleaseFound,
            current_version,
            latest_version: Some(release.tag_name),
            release_url: Some(release.html_url),
            selected_asset: None,
            downloaded_asset_path: None,
            last_checked_at: Some(Utc::now()),
            last_error: Some("Latest release is draft or prerelease.".into()),
        });
    }
    let latest_clean = release.tag_name.trim_start_matches('v');
    let current = Version::parse(&current_version)?;
    let latest = Version::parse(latest_clean)?;
    let selected = select_asset(release.assets).map(|asset| AppReleaseAsset {
        name: asset.name,
        browser_download_url: asset.browser_download_url,
        size: asset.size,
        content_type: asset.content_type,
    });
    let status = if latest <= current {
        AppUpdateStatusKind::UpToDate
    } else if selected.is_some() {
        AppUpdateStatusKind::UpdateAvailable
    } else {
        AppUpdateStatusKind::UnsupportedPlatformAsset
    };
    let state = AppUpdateState {
        status,
        current_version,
        latest_version: Some(latest.to_string()),
        release_url: Some(release.html_url),
        selected_asset: selected,
        downloaded_asset_path: None,
        last_checked_at: Some(Utc::now()),
        last_error: None,
    };
    std::fs::create_dir_all(&config.update_stage_dir)?;
    Ok(state)
}

pub fn download_selected(config: &AppConfig, state: &AppUpdateState) -> Result<AppUpdateState> {
    let asset = state
        .selected_asset
        .clone()
        .ok_or(SpiceError::MissingReleaseAsset)?;
    std::fs::create_dir_all(&config.update_stage_dir)?;
    let target = config.update_stage_dir.join(&asset.name);
    let client = reqwest::blocking::Client::builder()
        .user_agent("SpiceManager app-update")
        .build()?;
    let mut response = client
        .get(&asset.browser_download_url)
        .send()?
        .error_for_status()?;
    let mut file = std::fs::File::create(&target)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let read = response.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        std::io::Write::write_all(&mut file, &buffer[..read])?;
    }
    let sha = format!("{:x}", hasher.finalize());
    let mut next = state.clone();
    next.status = AppUpdateStatusKind::DownloadedPendingInstall;
    next.downloaded_asset_path = Some(target);
    next.last_error = None;
    next.last_checked_at = Some(Utc::now());
    next.release_url = next
        .release_url
        .clone()
        .or_else(|| Some(format!("sha256:{sha}")));
    Ok(next)
}

fn select_asset(assets: Vec<GithubAsset>) -> Option<GithubAsset> {
    let os = if cfg!(target_os = "windows") {
        ["windows", "win", ".msi", ".exe", ".nsis"].as_slice()
    } else if cfg!(target_os = "macos") {
        ["macos", "darwin", ".dmg", ".app.tar.gz"].as_slice()
    } else if cfg!(target_os = "linux") {
        ["linux", ".appimage", ".deb", ".rpm"].as_slice()
    } else {
        [].as_slice()
    };
    assets.into_iter().find(|asset| {
        let name = asset.name.to_lowercase();
        os.iter().any(|token| name.contains(token))
    })
}
