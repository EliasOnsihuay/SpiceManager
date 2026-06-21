import { invoke } from "@tauri-apps/api/core";
import type {
  AppUpdateState,
  DiagnosticsReport,
  EnvironmentState,
  StatusPayload,
  WorkflowReport,
} from "./types";

export async function currentStatus(): Promise<StatusPayload> {
  const [environment, appUpdate] = await invoke<[EnvironmentState | null, AppUpdateState]>(
    "current_status",
  );
  return { environment, appUpdate };
}

export async function runWorkflow(command: string): Promise<WorkflowReport> {
  return invoke<WorkflowReport>(command);
}

export async function doctorReport(): Promise<DiagnosticsReport> {
  return invoke<DiagnosticsReport>("doctor_report");
}

export async function exportDiagnostics(): Promise<string> {
  return invoke<string>("export_diagnostics");
}

export async function appUpdateCheck(): Promise<AppUpdateState> {
  return invoke<AppUpdateState>("app_update_check", { owner: null, repo: null });
}

export async function appUpdateDownload(): Promise<AppUpdateState> {
  return invoke<AppUpdateState>("app_update_download");
}
