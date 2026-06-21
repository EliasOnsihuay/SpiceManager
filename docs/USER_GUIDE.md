# SpiceManager User Guide

SpiceManager is a third-party desktop utility for managing Spotify + Spicetify setups. It is not an official Spotify or Spicetify product.

## First Run

1. Install Spotify classic desktop where possible.
2. Open SpiceManager.
3. Use Overview to inspect Spotify, Spicetify, Marketplace, adblock, compatibility, and app-update state.
4. Use Maintenance when you want SpiceManager to install, update, repair, validate, or recover the setup.
5. Use Diagnostics when something fails or when you need a report to share.

## Recommended Actions

- Run `Install` when Spicetify is missing.
- Run `Repair` when Marketplace or adblock is missing or broken.
- Run `Validate` after Spotify updates.
- Run `Recover` when compatibility hold mode is active and you are ready to try recovery.
- Run `Check app updates` to check SpiceManager releases on GitHub.

## Compatibility Hold Mode

Spotify updates can temporarily break Spicetify. When SpiceManager sees version changes, repeated apply failures, or managed feature regressions, it can enter hold mode. Hold mode prevents repeated apply attempts and keeps the last known good state available for diagnostics and recovery.

## Diagnostics Export

Diagnostics exports include:

- Environment summary
- Spotify, Spicetify, Marketplace, and adblock state
- Compatibility status and hold reason
- Last known good state
- Recent workflow reports
- Log excerpts
- App version and app-update status

Use this export when reporting a problem.
