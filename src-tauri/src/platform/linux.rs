use crate::errors::Result;
use crate::models::{SpotifyInstallKind, SpotifyState};
use std::path::PathBuf;

pub fn detect_spotify() -> Result<SpotifyState> {
    let apt_path = PathBuf::from("/usr/bin/spotify");
    let flatpak_path = PathBuf::from("/var/lib/flatpak/app/com.spotify.Client");
    let aur_path = PathBuf::from("/opt/spotify/spotify");
    let executable_path = [apt_path.clone(), aur_path.clone()]
        .into_iter()
        .find(|p| p.exists())
        .or_else(|| command_path("spotify"));
    let flatpak_installed = flatpak_path.exists()
        || crate::utils::shell::run("flatpak", &["info", "com.spotify.Client"]).success;
    let install_kind = if apt_path.exists() {
        SpotifyInstallKind::LinuxApt
    } else if flatpak_installed {
        SpotifyInstallKind::LinuxFlatpak
    } else if aur_path.exists() {
        SpotifyInstallKind::LinuxAur
    } else if executable_path.is_some() {
        SpotifyInstallKind::LinuxUnknown
    } else {
        SpotifyInstallKind::Unknown
    };
    let home = dirs::home_dir();
    let prefs_path = match install_kind {
        SpotifyInstallKind::LinuxFlatpak => home.as_ref().map(|h| {
            h.join(".var")
                .join("app")
                .join("com.spotify.Client")
                .join("config")
                .join("spotify")
                .join("prefs")
        }),
        _ => home
            .as_ref()
            .map(|h| h.join(".config").join("spotify").join("prefs")),
    };
    let resources_path = match install_kind {
        SpotifyInstallKind::LinuxFlatpak => home.as_ref().map(|h| {
            h.join(".var")
                .join("app")
                .join("com.spotify.Client")
                .join("config")
                .join("spotify")
                .join("Apps")
        }),
        _ => Some(PathBuf::from("/usr/share/spotify/Apps")),
    };
    let running = crate::utils::shell::run("pgrep", &["-x", "spotify"]).success;
    let version_result = crate::utils::shell::run("spotify", &["--version"]);
    let version = version_result
        .success
        .then_some(version_result.stdout)
        .filter(|s| !s.is_empty());
    let mut warnings = Vec::new();
    if matches!(install_kind, SpotifyInstallKind::LinuxUnknown) {
        warnings.push("Spotify exists, but the Linux package type is not recognized yet.".into());
    }
    Ok(SpotifyState {
        installed: executable_path.is_some() || flatpak_installed,
        install_kind: install_kind.clone(),
        version,
        executable_path,
        resources_path,
        prefs_path,
        running,
        likely_usable_for_spicetify: matches!(
            install_kind,
            SpotifyInstallKind::LinuxApt | SpotifyInstallKind::LinuxAur | SpotifyInstallKind::LinuxFlatpak
        ),
        warnings,
        notes: vec![],
    })
}

fn command_path(command: &str) -> Option<PathBuf> {
    let result = crate::utils::shell::run("which", &[command]);
    result.success.then(|| PathBuf::from(result.stdout.trim()))
}
