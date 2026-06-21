# Release Guide

SpiceManager releases are published through GitHub Releases.

## Local Verification

Run:

```powershell
npm run check
npm run tauri:build
```

On Windows, this produces:

- `src-tauri/target/release/bundle/nsis/SpiceManager_<version>_x64-setup.exe`
- `src-tauri/target/release/bundle/msi/SpiceManager_<version>_x64_en-US.msi`

## GitHub Release Flow

1. Update versions in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. Commit and push changes.
3. Create a tag like `v0.1.1`.
4. Push the tag.
5. The GitHub Actions release workflow builds platform artifacts and creates a draft release.
6. Review the draft release notes and publish.

## App Self-Update Alignment

The app-update client checks GitHub Releases for the configured owner and repo. Release assets should include platform-identifying names such as:

- `windows`, `win`, `.msi`, `.exe`, or `nsis`
- `macos`, `darwin`, `.dmg`, or `.app.tar.gz`
- `linux`, `.AppImage`, `.deb`, or `.rpm`

This naming keeps asset selection predictable for the in-app update checker.
