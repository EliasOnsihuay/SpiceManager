# SpiceManager

SpiceManager is a third-party desktop utility for managing Spotify + Spicetify setups on Windows, macOS, and Linux. It is authored by Elias Onsihuay and maintained by Kodhu Technologies.

SpiceManager is not an official Spotify or Spicetify product and is not affiliated with either project.

## Branding

- Product: SpiceManager
- Author / Lead Developer: Elias Onsihuay
- Maintained by: Kodhu Technologies
- Logo source: `src/assets/logo.svg`, also used to generate native Tauri icons in `src-tauri/icons`.

## What It Does

- Detects Spotify, Spicetify, Marketplace, rxri adblockify extension/configuration, and compatibility state.
- Installs, updates, repairs, and validates the Spotify + Spicetify ecosystem.
- Tracks Spotify updates that may break Spicetify and uses compatibility hold mode to avoid repeated destructive apply attempts.
- Treats Marketplace and rxri adblockify state as first-class managed features.
- Checks and stages SpiceManager app updates from GitHub Releases.
- Provides a Tauri desktop UI and a CLI for development, scripting, and diagnostics.

## Downloads

User-ready builds are published on GitHub Releases:

- [Latest SpiceManager release](https://github.com/EliasOnsihuay/SpiceManager/releases/latest)

Windows users can choose the NSIS setup executable or the MSI installer. macOS and Linux artifacts are prepared through the release workflow when tags are built on GitHub Actions.

For most Windows users, the NSIS setup executable is recommended because it creates the normal Start Menu/Desktop integration and launches the desktop app. The raw executable is useful only as a portable/dev artifact.

## Supported Platforms

- Windows: detects classic desktop Spotify and Microsoft Store Spotify. Classic desktop Spotify is preferred for Spicetify workflows. Store-only installs are reported with clear limitations.
- macOS: detects `/Applications/Spotify.app` and common user configuration/resource paths.
- Linux: detects APT/deb, Flatpak, and AUR-style Spotify installs where observable. Unknown packaging types are reported as unsupported or uncertain instead of treated as successful.

## Workflows

- `detect`: inspect the current environment and persist the latest state.
- `install`: install missing Spicetify pieces, Marketplace, and rxri adblockify configuration where possible.
- `update`: update Spicetify, repair Marketplace/adblock if needed, then validate.
- `repair`: run restore/backup/apply fallback flows and re-check managed state.
- `validate`: run a non-installing health check.
- `doctor`: export a structured report with warnings, logs, last known good state, compatibility state, and update status.

## Compatibility Hold Mode

Spotify can update before Spicetify supports the new version. SpiceManager stores the last known good Spotify + Spicetify state and recent apply failures. If a Spotify version change is followed by repeated failures or Marketplace/adblock regression, SpiceManager enters hold mode, preserves the desired managed state, and stops hammering `spicetify apply`. Recovery can later update Spicetify, repair Marketplace, restore adblock configuration, re-apply, validate, and clear hold mode.

## Adblockify

SpiceManager manages `adblock.js` from [`rxri/spicetify-extensions`](https://github.com/rxri/spicetify-extensions). The repair/install flow downloads the extension to the Spicetify `Extensions` folder, preserves an existing copy as `adblock.js.spicemanager.bak` when it changes, ensures `extensions = adblock.js`, and keeps Marketplace enabled through `custom_apps = marketplace`.

## App Self-Update

SpiceManager app updates are separate from Spotify/Spicetify ecosystem updates. The app self-update client checks GitHub Releases, compares semantic versions, selects platform-specific assets, downloads them to the local update staging directory, and persists status such as `UpToDate`, `UpdateAvailable`, `DownloadedPendingInstall`, or `CheckFailed`.

Configure the release source with:

```powershell
spicemanager-cli app-update check --owner <github-owner> --repo <github-repo>
```

## CLI

```powershell
spicemanager-cli detect
spicemanager-cli install
spicemanager-cli update
spicemanager-cli repair
spicemanager-cli validate
spicemanager-cli status
spicemanager-cli doctor
spicemanager-cli app-update check
spicemanager-cli app-update download
spicemanager-cli app-update status
```

CLI output clearly separates Spotify/Spicetify ecosystem state from SpiceManager app self-update state.

The installed desktop app is `spicemanager.exe`. The CLI/dev binary is intentionally named `spicemanager-cli.exe` so desktop shortcuts launch the Tauri app, not the terminal CLI.

## Development

Install dependencies:

```powershell
npm install
```

Run the desktop app:

```powershell
npm run tauri:dev
```

Run frontend only:

```powershell
npm run dev
```

Check Rust:

```powershell
npm run check:rust
```

Build release artifacts:

```powershell
npm run tauri:build
```

Release automation and manual publishing notes are in [`docs/RELEASE.md`](docs/RELEASE.md). A user guide is available in [`docs/USER_GUIDE.md`](docs/USER_GUIDE.md).

## State, Logs, and Diagnostics

SpiceManager stores JSON state, workflow reports, compatibility data, app update status, logs, and staged app updates in platform-local app data directories:

- Windows: `%APPDATA%\Kodhu Technologies\SpiceManager`
- macOS: `~/Library/Application Support/com.kodhu.spicemanager`
- Linux: `$XDG_CONFIG_HOME/spicemanager` or `~/.config/spicemanager`

Use the desktop Diagnostics section or:

```powershell
spicemanager-cli doctor
```

The diagnostics export includes environment summary, managed component state, compatibility hold reason, last known good state, recent workflow results, log excerpts, app version, app-update status, and timestamps.
