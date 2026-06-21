use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    NeedsAction,
    Hold,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpotifyInstallKind {
    Classic,
    MicrosoftStore,
    BothClassicPreferred,
    MacApplication,
    LinuxApt,
    LinuxFlatpak,
    LinuxAur,
    LinuxUnknown,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyState {
    pub installed: bool,
    pub install_kind: SpotifyInstallKind,
    pub version: Option<String>,
    pub executable_path: Option<PathBuf>,
    pub resources_path: Option<PathBuf>,
    pub prefs_path: Option<PathBuf>,
    pub running: bool,
    pub likely_usable_for_spicetify: bool,
    pub warnings: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpicetifyState {
    pub installed: bool,
    pub version: Option<String>,
    pub binary_path: Option<PathBuf>,
    pub config_path: Option<PathBuf>,
    pub configured: bool,
    pub last_apply_summary: Option<String>,
    pub warnings: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ManagedFeatureState {
    Installed,
    Missing,
    Broken,
    Configured,
    Failed,
    Uncertain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceState {
    pub state: ManagedFeatureState,
    pub path: Option<PathBuf>,
    pub warnings: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdblockState {
    pub state: ManagedFeatureState,
    pub extension_name: String,
    pub config_entry_present: bool,
    pub warnings: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompatibilityStatus {
    Compatible,
    Unknown,
    NeedsValidation,
    LikelyBrokenAfterSpotifyUpdate,
    NeedsSpicetifyUpdate,
    TemporarilyBlockedByUpstream,
    HoldModeActive,
    RecoveryInProgress,
    RecoveredPendingValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastKnownGoodState {
    pub recorded_at: DateTime<Utc>,
    pub spotify_version: Option<String>,
    pub spicetify_version: Option<String>,
    pub marketplace_state: ManagedFeatureState,
    pub adblock_state: ManagedFeatureState,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityState {
    pub status: CompatibilityStatus,
    pub hold_mode_active: bool,
    pub reason: Option<String>,
    pub current_spotify_version: Option<String>,
    pub current_spicetify_version: Option<String>,
    pub previous_spotify_version: Option<String>,
    pub previous_spicetify_version: Option<String>,
    pub last_known_good: Option<LastKnownGoodState>,
    pub recent_apply_failures: u32,
    pub last_recovery_attempt: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentState {
    pub platform: Platform,
    pub detected_at: DateTime<Utc>,
    pub overall_health: HealthStatus,
    pub spotify: SpotifyState,
    pub spicetify: SpicetifyState,
    pub marketplace: MarketplaceState,
    pub adblock: AdblockState,
    pub compatibility: CompatibilityState,
    pub recommended_next_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellCommandResult {
    pub command: String,
    pub args: Vec<String>,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyAttempt {
    pub label: String,
    pub result: ShellCommandResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub attempted: Vec<ApplyAttempt>,
    pub success: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowKind {
    Detect,
    Install,
    Update,
    Repair,
    Validate,
    Doctor,
    CompatibilityRecovery,
    AppUpdateCheck,
    AppUpdateDownload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowReport {
    pub id: Uuid,
    pub kind: WorkflowKind,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub success: bool,
    pub summary: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub apply_result: Option<ApplyResult>,
    pub environment: Option<EnvironmentState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppUpdateStatusKind {
    UpToDate,
    UpdateAvailable,
    NoReleaseFound,
    UnsupportedPlatformAsset,
    DownloadInProgress,
    DownloadedPendingInstall,
    InstallScheduled,
    InstallFailed,
    CheckFailed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUpdateState {
    pub status: AppUpdateStatusKind,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub release_url: Option<String>,
    pub selected_asset: Option<AppReleaseAsset>,
    pub downloaded_asset_path: Option<PathBuf>,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    pub generated_at: DateTime<Utc>,
    pub app_version: String,
    pub environment: Option<EnvironmentState>,
    pub app_update: AppUpdateState,
    pub recent_workflows: Vec<WorkflowReport>,
    pub log_excerpt: Vec<String>,
    pub state_dir: PathBuf,
    pub log_file: PathBuf,
}

impl Default for CompatibilityState {
    fn default() -> Self {
        Self {
            status: CompatibilityStatus::Unknown,
            hold_mode_active: false,
            reason: None,
            current_spotify_version: None,
            current_spicetify_version: None,
            previous_spotify_version: None,
            previous_spicetify_version: None,
            last_known_good: None,
            recent_apply_failures: 0,
            last_recovery_attempt: None,
            updated_at: Utc::now(),
        }
    }
}

impl Default for AppUpdateState {
    fn default() -> Self {
        Self {
            status: AppUpdateStatusKind::Unknown,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            latest_version: None,
            release_url: None,
            selected_asset: None,
            downloaded_asset_path: None,
            last_checked_at: None,
            last_error: None,
        }
    }
}
