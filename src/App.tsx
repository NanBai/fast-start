import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  CLI_LABELS,
  CLI_ORDER,
  CliScanError,
  CliType,
  ScanResponse,
  SessionData,
  TERMINAL_LABELS,
  TerminalType,
} from "./types";
import "./App.css";

type StatusType = "info" | "success" | "error";

function formatTime(iso: string) {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return iso;
  }
  return date.toLocaleString();
}

function TerminalSelector({
  value,
  available,
  onChange,
}: {
  value: TerminalType;
  available: TerminalType[];
  onChange: (terminal: TerminalType) => Promise<void>;
}) {
  const [saving, setSaving] = useState(false);

  async function handleChange(next: TerminalType) {
    setSaving(true);
    try {
      await onChange(next);
    } finally {
      setSaving(false);
    }
  }

  return (
    <label className="terminal-selector">
      <span>终端</span>
      <select
        value={value}
        disabled={saving}
        onChange={(event) => handleChange(event.target.value as TerminalType)}
      >
        {(Object.keys(TERMINAL_LABELS) as TerminalType[]).map((terminal) => {
          const enabled = available.includes(terminal);
          return (
            <option key={terminal} value={terminal} disabled={!enabled}>
              {TERMINAL_LABELS[terminal]}
              {!enabled ? "（未安装）" : ""}
            </option>
          );
        })}
      </select>
    </label>
  );
}

function SessionRow({
  session,
  launchingId,
  onLaunch,
}: {
  session: SessionData;
  launchingId: string | null;
  onLaunch: (sessionId: string) => Promise<void>;
}) {
  return (
    <div className="session-row">
      <div className="session-main">
        <strong>{session.projectName}</strong>
        <span className="session-path">{session.projectDir}</span>
        <span className="session-time">{formatTime(session.lastActiveAt)}</span>
      </div>
      <button
        type="button"
        disabled={launchingId === session.id}
        onClick={() => onLaunch(session.id)}
      >
        {launchingId === session.id ? "启动中..." : "启动"}
      </button>
    </div>
  );
}

function CliGroup({
  cliType,
  sessions,
  scanError,
  launchingId,
  onLaunch,
}: {
  cliType: CliType;
  sessions: SessionData[];
  scanError?: CliScanError;
  launchingId: string | null;
  onLaunch: (sessionId: string) => Promise<void>;
}) {
  const [expanded, setExpanded] = useState(true);

  let body = null;
  if (cliType === "cursor") {
    body = <p className="group-empty">cursor 支持开发中（v2）</p>;
  } else if (scanError) {
    body = <p className="group-error">扫描失败：{scanError.message}</p>;
  } else if (sessions.length === 0) {
    body = <p className="group-empty">暂无 session</p>;
  } else {
    body = sessions.map((session) => (
      <SessionRow
        key={session.id}
        session={session}
        launchingId={launchingId}
        onLaunch={onLaunch}
      />
    ));
  }

  return (
    <section className="cli-group">
      <button
        type="button"
        className="cli-group-header"
        onClick={() => setExpanded((current) => !current)}
      >
        <span>{CLI_LABELS[cliType]}</span>
        <span className="cli-group-meta">
          {cliType === "cursor"
            ? "v2"
            : scanError
              ? "失败"
              : `${sessions.length} 条`}
        </span>
      </button>
      {expanded ? <div className="cli-group-body">{body}</div> : null}
    </section>
  );
}

function App() {
  const [sessions, setSessions] = useState<SessionData[]>([]);
  const [scanErrors, setScanErrors] = useState<CliScanError[]>([]);
  const [availableTerminals, setAvailableTerminals] = useState<TerminalType[]>([
    "system",
  ]);
  const [preferredTerminal, setPreferredTerminal] =
    useState<TerminalType>("system");
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [launchingId, setLaunchingId] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string>("");
  const [statusType, setStatusType] = useState<StatusType>("info");

  async function loadTerminals() {
    const [available, preferred] = await Promise.all([
      invoke<TerminalType[]>("list_available_terminals"),
      invoke<TerminalType>("get_preferred_terminal"),
    ]);
    setAvailableTerminals(available);
    const resolved = available.includes(preferred)
      ? preferred
      : (available[0] ?? "system");
    setPreferredTerminal(resolved);
    if (resolved !== preferred) {
      await invoke("set_preferred_terminal", { terminal: resolved });
    }
  }

  async function applyScanResult(result: ScanResponse) {
    setSessions(result.sessions);
    setScanErrors(result.scanErrors);
    setStatusMessage(
      result.scanErrors.length > 0
        ? `已加载 ${result.sessions.length} 条 session，${result.scanErrors.length} 个 CLI 扫描失败`
        : `已加载 ${result.sessions.length} 条 session`,
    );
    setStatusType(result.scanErrors.length > 0 ? "error" : "info");
  }

  async function loadSessions() {
    setLoading(true);
    try {
      const result = await invoke<ScanResponse>("scan_sessions");
      await applyScanResult(result);
    } catch (error) {
      setStatusMessage(String(error));
      setStatusType("error");
    } finally {
      setLoading(false);
    }
  }

  async function refreshSessions() {
    setRefreshing(true);
    try {
      const result = await invoke<ScanResponse>("refresh_sessions");
      await applyScanResult(result);
    } catch (error) {
      setStatusMessage(String(error));
      setStatusType("error");
    } finally {
      setRefreshing(false);
    }
  }

  async function handleTerminalChange(terminal: TerminalType) {
    await invoke("set_preferred_terminal", { terminal });
    setPreferredTerminal(terminal);
    setStatusMessage(`已切换终端为 ${TERMINAL_LABELS[terminal]}`);
    setStatusType("info");
  }

  async function handleLaunch(sessionId: string) {
    setLaunchingId(sessionId);
    setStatusMessage("正在启动终端...");
    setStatusType("info");
    try {
      await invoke("launch_session", { sessionId });
      setStatusMessage("终端启动成功");
      setStatusType("success");
    } catch (error) {
      setStatusMessage(`启动失败：${String(error)}`);
      setStatusType("error");
    } finally {
      setLaunchingId(null);
    }
  }

  useEffect(() => {
    void (async () => {
      await loadTerminals();
      await loadSessions();
    })();
  }, []);

  const grouped = new Map<CliType, SessionData[]>();
  for (const cliType of CLI_ORDER) {
    grouped.set(cliType, []);
  }
  for (const session of sessions) {
    grouped.get(session.cliType)?.push(session);
  }

  return (
    <main className="app-shell">
      <header className="app-header">
        <div>
          <h1>Session Launcher</h1>
          <p className="app-subtitle">聚合 codex / claude-code session，一键恢复工作现场</p>
        </div>
        <div className="header-actions">
          <TerminalSelector
            value={preferredTerminal}
            available={availableTerminals}
            onChange={handleTerminalChange}
          />
          <button type="button" onClick={() => void refreshSessions()} disabled={refreshing}>
            {refreshing ? "刷新中..." : "刷新"}
          </button>
        </div>
      </header>

      <p className={`status-bar status-${statusType}`} aria-live="polite">
        {loading ? "正在扫描 session..." : statusMessage}
      </p>

      <div className="session-list">
        {CLI_ORDER.map((cliType) => (
          <CliGroup
            key={cliType}
            cliType={cliType}
            sessions={grouped.get(cliType) ?? []}
            scanError={scanErrors.find((item) => item.cliType === cliType)}
            launchingId={launchingId}
            onLaunch={handleLaunch}
          />
        ))}
      </div>
    </main>
  );
}

export default App;
