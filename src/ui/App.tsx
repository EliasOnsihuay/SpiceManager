import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  Download,
  FileDown,
  Hammer,
  Info,
  RefreshCcw,
  RotateCcw,
  ShieldAlert,
  Terminal,
  Wrench,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import logoUrl from "../assets/logo.svg";
import {
  appUpdateCheck,
  appUpdateDownload,
  currentStatus,
  doctorReport,
  exportDiagnostics,
  runWorkflow,
} from "../api";
import type { AppUpdateState, DiagnosticsReport, EnvironmentState, WorkflowReport } from "../types";

type Tab = "dashboard" | "actions" | "diagnostics" | "settings";

const workflowCommands = {
  detect: "detect_state",
  install: "install_all",
  update: "update_all",
  repair: "repair_all",
  validate: "validate_all",
  recover: "recover_compatibility",
};

export function App() {
  const [tab, setTab] = useState<Tab>("dashboard");
  const [environment, setEnvironment] = useState<EnvironmentState | null>(null);
  const [appUpdate, setAppUpdate] = useState<AppUpdateState | null>(null);
  const [diagnostics, setDiagnostics] = useState<DiagnosticsReport | null>(null);
  const [lastWorkflow, setLastWorkflow] = useState<WorkflowReport | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [exportPath, setExportPath] = useState<string | null>(null);

  async function refresh() {
    setBusy("Refreshing");
    setError(null);
    try {
      const status = await currentStatus();
      setEnvironment(status.environment);
      setAppUpdate(status.appUpdate);
      if (!status.environment) {
        const report = await runWorkflow(workflowCommands.detect);
        setLastWorkflow(report);
        setEnvironment(report.environment ?? null);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(null);
    }
  }

  async function run(label: string, command: string) {
    setBusy(label);
    setError(null);
    setExportPath(null);
    try {
      const report = await runWorkflow(command);
      setLastWorkflow(report);
      setEnvironment(report.environment ?? null);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(null);
    }
  }

  async function loadDoctor() {
    setBusy("Diagnostics");
    setError(null);
    try {
      const report = await doctorReport();
      setDiagnostics(report);
      setEnvironment(report.environment ?? null);
      setAppUpdate(report.app_update);
      setTab("diagnostics");
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(null);
    }
  }

  async function checkUpdates() {
    setBusy("App update");
    setError(null);
    try {
      setAppUpdate(await appUpdateCheck());
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(null);
    }
  }

  async function downloadUpdate() {
    setBusy("Download update");
    setError(null);
    try {
      setAppUpdate(await appUpdateDownload());
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(null);
    }
  }

  async function exportReport() {
    setBusy("Export diagnostics");
    setError(null);
    try {
      setExportPath(await exportDiagnostics());
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(null);
    }
  }

  useEffect(() => {
    refresh();
  }, []);

  const healthClass = useMemo(() => statusClass(environment?.overall_health), [environment]);

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <img src={logoUrl} alt="SpiceManager logo" />
          <div>
            <h1>SpiceManager</h1>
            <p>Third-party Spotify + Spicetify utility</p>
          </div>
        </div>
        <nav>
          <button className={tab === "dashboard" ? "active" : ""} onClick={() => setTab("dashboard")}>
            <Activity size={18} /> Overview
          </button>
          <button className={tab === "actions" ? "active" : ""} onClick={() => setTab("actions")}>
            <Wrench size={18} /> Maintenance
          </button>
          <button className={tab === "diagnostics" ? "active" : ""} onClick={loadDoctor}>
            <Terminal size={18} /> Diagnostics
          </button>
          <button className={tab === "settings" ? "active" : ""} onClick={() => setTab("settings")}>
            <Info size={18} /> Settings
          </button>
        </nav>
      </aside>

      <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">Maintained by Kodhu Technologies</p>
            <h2>{titleFor(tab)}</h2>
          </div>
          <div className={`health ${healthClass}`}>
            {environment?.compatibility.hold_mode_active ? <ShieldAlert size={18} /> : <CheckCircle2 size={18} />}
            {environment?.overall_health ?? "Unknown"}
          </div>
        </header>

        {error && <div className="notice error">{error}</div>}
        {busy && <div className="notice busy">{busy} in progress...</div>}
        {exportPath && <div className="notice success">Diagnostics exported to {exportPath}</div>}

        {tab === "dashboard" && (
          <Dashboard environment={environment} appUpdate={appUpdate} onRefresh={() => run("Detect", workflowCommands.detect)} />
        )}
        {tab === "actions" && (
          <Actions
            busy={Boolean(busy)}
            hold={Boolean(environment?.compatibility.hold_mode_active)}
            onRun={run}
            onDoctor={loadDoctor}
            onCheckUpdates={checkUpdates}
            onDownloadUpdate={downloadUpdate}
          />
        )}
        {tab === "diagnostics" && (
          <Diagnostics
            diagnostics={diagnostics}
            lastWorkflow={lastWorkflow}
            environment={environment}
            onRefresh={loadDoctor}
            onExport={exportReport}
          />
        )}
        {tab === "settings" && <Settings appUpdate={appUpdate} environment={environment} />}
      </section>
    </main>
  );
}

function Dashboard({
  environment,
  appUpdate,
  onRefresh,
}: {
  environment: EnvironmentState | null;
  appUpdate: AppUpdateState | null;
  onRefresh: () => void;
}) {
  return (
    <div className="content-grid">
      <section className="overview-band">
        <div>
          <p className="eyebrow">Overall</p>
          <h3>{environment?.recommended_next_action ?? "Run detection to build the first environment snapshot."}</h3>
        </div>
        <button className="primary" onClick={onRefresh}>
          <RefreshCcw size={18} /> Refresh
        </button>
      </section>
      {environment?.compatibility.hold_mode_active && (
        <section className="hold-band">
          <ShieldAlert size={24} />
          <div>
            <h3>Compatibility hold active</h3>
            <p>{environment.compatibility.reason}</p>
          </div>
        </section>
      )}
      <div className="status-grid">
        <StatusPanel title="Platform" value={formatPlatform(environment?.platform)} detail={environment?.detected_at} />
        <StatusPanel
          title="Spotify"
          value={environment?.spotify.installed ? environment.spotify.install_kind : "Not installed"}
          detail={environment?.spotify.version ?? environment?.spotify.executable_path ?? "No version detected"}
          warnings={environment?.spotify.warnings}
        />
        <StatusPanel
          title="Spicetify"
          value={environment?.spicetify.installed ? "Installed" : "Missing"}
          detail={environment?.spicetify.version ?? environment?.spicetify.config_path ?? "No config detected"}
          warnings={environment?.spicetify.warnings}
        />
        <StatusPanel
          title="Marketplace"
          value={environment?.marketplace.state ?? "Unknown"}
          detail={environment?.marketplace.path ?? "No Marketplace path detected"}
          warnings={environment?.marketplace.warnings}
        />
        <StatusPanel
          title="Adblock"
          value={environment?.adblock.state ?? "Unknown"}
          detail={environment?.adblock.config_entry_present ? "Config entry present" : "No config entry detected"}
          warnings={environment?.adblock.warnings}
        />
        <StatusPanel
          title="App Update"
          value={appUpdate?.status ?? "Unknown"}
          detail={appUpdate?.latest_version ? `Latest ${appUpdate.latest_version}` : `Current ${appUpdate?.current_version ?? "0.1.0"}`}
          warnings={appUpdate?.last_error ? [appUpdate.last_error] : []}
        />
      </div>
    </div>
  );
}

function Actions({
  busy,
  hold,
  onRun,
  onDoctor,
  onCheckUpdates,
  onDownloadUpdate,
}: {
  busy: boolean;
  hold: boolean;
  onRun: (label: string, command: string) => void;
  onDoctor: () => void;
  onCheckUpdates: () => void;
  onDownloadUpdate: () => void;
}) {
  const actions = [
    ["Install", workflowCommands.install, <Download size={18} />],
    ["Update", workflowCommands.update, <RefreshCcw size={18} />],
    ["Repair", workflowCommands.repair, <Hammer size={18} />],
    ["Validate", workflowCommands.validate, <CheckCircle2 size={18} />],
  ] as const;
  return (
    <div className="action-layout">
      <section className="action-strip">
        {actions.map(([label, command, icon]) => (
          <button key={command} disabled={busy} onClick={() => onRun(label, command)}>
            {icon} {label}
          </button>
        ))}
        <button disabled={busy || !hold} onClick={() => onRun("Compatibility recovery", workflowCommands.recover)}>
          <RotateCcw size={18} /> Recover
        </button>
      </section>
      <section className="action-strip secondary">
        <button disabled={busy} onClick={onDoctor}>
          <Terminal size={18} /> Doctor
        </button>
        <button disabled={busy} onClick={onCheckUpdates}>
          <RefreshCcw size={18} /> Check app updates
        </button>
        <button disabled={busy} onClick={onDownloadUpdate}>
          <Download size={18} /> Stage app update
        </button>
      </section>
    </div>
  );
}

function Diagnostics({
  diagnostics,
  lastWorkflow,
  environment,
  onRefresh,
  onExport,
}: {
  diagnostics: DiagnosticsReport | null;
  lastWorkflow: WorkflowReport | null;
  environment: EnvironmentState | null;
  onRefresh: () => void;
  onExport: () => void;
}) {
  const workflows = diagnostics?.recent_workflows ?? (lastWorkflow ? [lastWorkflow] : []);
  return (
    <div className="diagnostics-layout">
      <section className="overview-band">
        <div>
          <p className="eyebrow">Doctor</p>
          <h3>{environment?.compatibility.reason ?? "Diagnostics include state, recent workflows, and log excerpts."}</h3>
        </div>
        <div className="inline-actions">
          <button onClick={onRefresh}>
            <RefreshCcw size={18} /> Refresh
          </button>
          <button onClick={onExport}>
            <FileDown size={18} /> Export
          </button>
        </div>
      </section>
      <section className="workflow-list">
        {workflows.length === 0 && <article className="workflow">No workflow history yet.</article>}
        {workflows.slice(-8).reverse().map((workflow) => (
          <article key={workflow.id} className={workflow.success ? "workflow ok" : "workflow bad"}>
            <strong>{workflow.kind}</strong>
            <span>{workflow.summary}</span>
            {[...workflow.warnings, ...workflow.errors].map((item) => (
              <p key={item}>{item}</p>
            ))}
          </article>
        ))}
      </section>
      <pre className="log-box">{(diagnostics?.log_excerpt ?? []).join("\n") || "No log excerpt loaded yet."}</pre>
    </div>
  );
}

function Settings({ appUpdate, environment }: { appUpdate: AppUpdateState | null; environment: EnvironmentState | null }) {
  return (
    <div className="settings-list">
      <StatusPanel title="Product" value="SpiceManager" detail="Third-party utility for managing Spotify + Spicetify setups" />
      <StatusPanel title="Author" value="Elias Onsihuay" detail="Maintained by Kodhu Technologies" />
      <StatusPanel title="Version" value={appUpdate?.current_version ?? "0.1.0"} detail="GitHub Releases self-update enabled" />
      <StatusPanel title="Compatibility" value={environment?.compatibility.status ?? "Unknown"} detail={environment?.compatibility.reason ?? "No active hold reason"} />
      <StatusPanel title="Update Asset" value={appUpdate?.selected_asset?.name ?? "No asset selected"} detail={appUpdate?.downloaded_asset_path ?? appUpdate?.release_url ?? "Check for updates to select an asset"} />
    </div>
  );
}

function StatusPanel({
  title,
  value,
  detail,
  warnings = [],
}: {
  title: string;
  value?: string | null;
  detail?: string | null;
  warnings?: string[];
}) {
  return (
    <article className="status-panel">
      <p className="eyebrow">{title}</p>
      <h3>{value ?? "Unknown"}</h3>
      <span>{detail ?? "No details available"}</span>
      {warnings.length > 0 && (
        <div className="warnings">
          <AlertTriangle size={16} />
          <p>{warnings[0]}</p>
        </div>
      )}
    </article>
  );
}

function statusClass(status?: string) {
  if (status === "Healthy") return "ok";
  if (status === "Hold") return "hold";
  if (status === "Failed" || status === "NeedsAction") return "bad";
  return "warn";
}

function formatPlatform(platform?: EnvironmentState["platform"]) {
  if (!platform) return "Unknown";
  if (typeof platform === "string") return platform;
  return platform.Unknown ?? "Unknown";
}

function titleFor(tab: Tab) {
  return {
    dashboard: "Overview",
    actions: "Maintenance",
    diagnostics: "Diagnostics",
    settings: "Settings / About",
  }[tab];
}
