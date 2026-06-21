# Privacy

SpiceManager stores state locally on the user's machine. The first serious version does not require Supabase, telemetry, or any hosted backend.

Local data can include:

- Detected Spotify and Spicetify paths and versions
- Marketplace and adblock state
- Compatibility status and last known good state
- Workflow reports
- Log excerpts
- App update check/download state

Network requests are limited to user-triggered install/update actions and GitHub Releases checks or downloads. Diagnostics exports are written locally and are not uploaded automatically.
