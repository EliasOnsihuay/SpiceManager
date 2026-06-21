use crate::errors::Result;
use crate::models::{SpotifyInstallKind, SpotifyState};
use std::path::PathBuf;

pub fn detect_spotify() -> Result<SpotifyState> {
    let app_path = PathBuf::from("/Applications/Spotify.app");
    let executable_path = app_path.join("Contents").join("MacOS").join("Spotify");
    let installed = app_path.exists();
    let home = dirs::home_dir();
    let prefs_path = home.as_ref().map(|h| {
        h.join("Library")
            .join("Application Support")
            .join("Spotify")
            .join("prefs")
    });
    let resources_path = Some(app_path.join("Contents").join("Resources"));
    let running = crate::utils::shell::run("pgrep", &["-x", "Spotify"]).success;
    let version = if installed {
        let plist = app_path.join("Contents").join("Info.plist");
        let result = crate::utils::shell::run(
            "defaults",
            &[
                "read",
                plist.to_string_lossy().as_ref(),
                "CFBundleShortVersionString",
            ],
        );
        result
            .success
            .then(|| result.stdout)
            .filter(|s| !s.is_empty())
    } else {
        None
    };
    Ok(SpotifyState {
        installed,
        install_kind: if installed {
            SpotifyInstallKind::MacApplication
        } else {
            SpotifyInstallKind::Unknown
        },
        version,
        executable_path: installed.then_some(executable_path),
        resources_path,
        prefs_path,
        running,
        likely_usable_for_spicetify: installed,
        warnings: vec![],
        notes: vec![],
    })
}
