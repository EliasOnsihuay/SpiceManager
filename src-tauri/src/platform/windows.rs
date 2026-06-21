use crate::errors::Result;
use crate::models::{SpotifyInstallKind, SpotifyState};
use crate::utils::fs::first_existing;
use std::path::PathBuf;

pub fn detect_spotify() -> Result<SpotifyState> {
    let appdata = std::env::var_os("APPDATA").map(PathBuf::from);
    let localappdata = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let classic_exe = first_existing(
        [appdata
            .as_ref()
            .map(|p| p.join("Spotify").join("Spotify.exe"))]
        .into_iter()
        .flatten(),
    );
    let store_shim = first_existing(
        [localappdata
            .as_ref()
            .map(|p| p.join("Microsoft").join("WindowsApps").join("Spotify.exe"))]
        .into_iter()
        .flatten(),
    );
    let store_marker = localappdata
        .as_ref()
        .map(|p| {
            p.join("Packages")
                .join("SpotifyAB.SpotifyMusic_zpdnekdrzrea0")
        })
        .filter(|p| p.exists());
    let classic_real = classic_exe.is_some();
    let store_present = store_marker.is_some() || store_shim.is_some();
    let installed = classic_real || store_present;
    let install_kind = match (classic_real, store_present) {
        (true, true) => SpotifyInstallKind::BothClassicPreferred,
        (true, false) => SpotifyInstallKind::Classic,
        (false, true) => SpotifyInstallKind::MicrosoftStore,
        _ => SpotifyInstallKind::Unknown,
    };
    let prefs_path = appdata.as_ref().map(|p| p.join("Spotify").join("prefs"));
    let resources_path = appdata.as_ref().map(|p| p.join("Spotify").join("Apps"));
    let running = crate::utils::shell::run("tasklist", &["/FI", "IMAGENAME eq Spotify.exe"])
        .stdout
        .contains("Spotify.exe");
    let mut warnings = Vec::new();
    if matches!(install_kind, SpotifyInstallKind::MicrosoftStore) {
        warnings.push(
            "Microsoft Store Spotify was detected. Spicetify workflows are limited; install classic desktop Spotify for full support.".into(),
        );
    }
    if installed && !classic_real {
        warnings.push("No classic Spotify install was found.".into());
    }
    Ok(SpotifyState {
        installed,
        install_kind,
        version: detect_version(classic_exe.as_ref()),
        executable_path: classic_exe,
        resources_path,
        prefs_path,
        running,
        likely_usable_for_spicetify: classic_real,
        warnings,
        notes: vec![],
    })
}

fn detect_version(exe: Option<&PathBuf>) -> Option<String> {
    exe.and_then(|path| {
        let escaped = path.to_string_lossy().replace('\'', "''");
        let result = crate::utils::shell::run(
            "powershell",
            &[
                "-NoProfile",
                "-Command",
                &format!(
                    "(Get-Item -LiteralPath '{}').VersionInfo.ProductVersion",
                    escaped
                ),
            ],
        );
        result
            .success
            .then(|| result.stdout)
            .filter(|s| !s.is_empty())
    })
}
