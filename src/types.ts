export type HealthStatus = "Healthy" | "Warning" | "NeedsAction" | "Hold" | "Failed" | "Unknown";

export interface SpotifyState {
  installed: boolean;
  install_kind: string;
  version?: string | null;
  executable_path?: string | null;
  resources_path?: string | null;
  prefs_path?: string | null;
  running: boolean;
  likely_usable_for_spicetify: boolean;
  warnings: string[];
  notes: string[];
}

export interface SpicetifyState {
  installed: boolean;
  version?: string | null;
  binary_path?: string | null;
  config_path?: string | null;
  configured: boolean;
  last_apply_summary?: string | null;
  warnings: string[];
  notes: string[];
}

export interface ManagedState {
  state: string;
  path?: string | null;
  warnings: string[];
  notes: string[];
}

export interface AdblockState {
  state: string;
  extension_name: string;
  config_entry_present: boolean;
  warnings: string[];
  notes: string[];
}

export interface CompatibilityState {
  status: string;
  hold_mode_active: boolean;
  reason?: string | null;
  current_spotify_version?: string | null;
  current_spicetify_version?: string | null;
  previous_spotify_version?: string | null;
  previous_spicetify_version?: string | null;
  recent_apply_failures: number;
  last_recovery_attempt?: string | null;
  updated_at: string;
}

export interface EnvironmentState {
  platform: string | { Unknown: string };
  detected_at: string;
  overall_health: HealthStatus;
  spotify: SpotifyState;
  spicetify: SpicetifyState;
  marketplace: ManagedState;
  adblock: AdblockState;
  compatibility: CompatibilityState;
  recommended_next_action?: string | null;
}

export interface AppReleaseAsset {
  name: string;
  browser_download_url: string;
  size: number;
  content_type?: string | null;
}

export interface AppUpdateState {
  status: string;
  current_version: string;
  latest_version?: string | null;
  release_url?: string | null;
  selected_asset?: AppReleaseAsset | null;
  downloaded_asset_path?: string | null;
  last_checked_at?: string | null;
  last_error?: string | null;
}

export interface ShellCommandResult {
  command: string;
  args: string[];
  stdout: string;
  stderr: string;
  exit_code?: number | null;
  success: boolean;
}

export interface ApplyAttempt {
  label: string;
  result: ShellCommandResult;
}

export interface ApplyResult {
  attempted: ApplyAttempt[];
  success: boolean;
  summary: string;
}

export interface WorkflowReport {
  id: string;
  kind: string;
  started_at: string;
  ended_at: string;
  success: boolean;
  summary: string;
  warnings: string[];
  errors: string[];
  apply_result?: ApplyResult | null;
  environment?: EnvironmentState | null;
}

export interface DiagnosticsReport {
  generated_at: string;
  app_version: string;
  environment?: EnvironmentState | null;
  app_update: AppUpdateState;
  recent_workflows: WorkflowReport[];
  log_excerpt: string[];
  state_dir: string;
  log_file: string;
}

export interface StatusPayload {
  environment: EnvironmentState | null;
  appUpdate: AppUpdateState;
}
