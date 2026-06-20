import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AgentGroup } from "./components/AgentGroup";
import {
  LaunchSegmented,
  RecentDaysMenu,
  TerminalMenu,
  ThemeMenu,
} from "./components/Controls";
import { Icon } from "./components/icons/Icon";
import { Skeleton } from "./components/Skeleton";
import {
  filterSessionsByRecentDays,
  RecentDaysFilter,
} from "./lib/sessionUtils";
import {
  CLI_LABELS,
  CLI_ORDER,
  CliScanError,
  CliType,
  LAUNCH_MODE_LABELS,
  LaunchMode,
  ScanResponse,
  SessionData,
  TERMINAL_LABELS,
  TerminalType,
  THEME_MODE_LABELS,
  ThemeMode,
} from "./types";
import "./App.css";

type StatusType = "info" | "success" | "error";

function applyThemeMode(mode: ThemeMode) {
  const root = document.documentElement;
  if (mode === "system") {
    delete root.dataset.theme;
  } else {
    root.dataset.theme = mode;
  }
}

function App() {
  const [sessions, setSessions] = useState<SessionData[]>([]);
  const [scanErrors, setScanErrors] = useState<CliScanError[]>([]);
  const [availableTerminals, setAvailableTerminals] = useState<TerminalType[]>([
    "system",
  ]);
  const [preferredTerminal, setPreferredTerminal] =
    useState<TerminalType>("system");
  const [launchMode, setLaunchMode] = useState<LaunchMode>("new-tab");
  const [themeMode, setThemeMode] = useState<ThemeMode>("system");
  const [recentDays, setRecentDays] = useState<RecentDaysFilter>("7");
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [launchingId, setLaunchingId] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string>("");
  const [statusType, setStatusType] = useState<StatusType>("info");

  async function loadPreferences() {
    const [available, preferred, mode, theme] = await Promise.all([
      invoke<TerminalType[]>("list_available_terminals"),
      invoke<TerminalType>("get_preferred_terminal"),
      invoke<LaunchMode>("get_launch_mode"),
      invoke<ThemeMode>("get_theme_mode"),
    ]);
    setAvailableTerminals(available);
    const resolved = available.includes(preferred)
      ? preferred
      : (available[0] ?? "system");
    setPreferredTerminal(resolved);
    if (resolved !== preferred) {
      await invoke("set_preferred_terminal", { terminal: resolved });
    }
    setLaunchMode(mode);
    setThemeMode(theme);
  }

  async function applyScanResult(result: ScanResponse) {
    setSessions(result.sessions);
    setScanErrors(result.scanErrors);
    setStatusMessage(
      result.scanErrors.length > 0
        ? `已加载 ${result.sessions.length} 个 session · ${result.scanErrors.length} 个 CLI 扫描失败`
        : `已加载 ${result.sessions.length} 个 session`,
    );
    setStatusType(result.scanErrors.length > 0 ? "error" : "success");
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
    setStatusMessage(`终端已切换为 ${TERMINAL_LABELS[terminal]}`);
    setStatusType("info");
  }

  async function handleLaunchModeChange(mode: LaunchMode) {
    await invoke("set_launch_mode", { mode });
    setLaunchMode(mode);
    setStatusMessage(`打开方式已切换为${LAUNCH_MODE_LABELS[mode]}`);
    setStatusType("info");
  }

  async function handleThemeModeChange(mode: ThemeMode) {
    await invoke("set_theme_mode", { mode });
    setThemeMode(mode);
    setStatusMessage(`主题已切换为${THEME_MODE_LABELS[mode]}`);
    setStatusType("info");
  }

  async function handleLaunch(sessionId: string) {
    setLaunchingId(sessionId);
    setStatusMessage("正在启动终端…");
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
      await loadPreferences();
      await loadSessions();
    })();
  }, []);

  useEffect(() => {
    applyThemeMode(themeMode);
  }, [themeMode]);

  // 先按 agent 分区；每个 agent 内部再按工作目录聚合。
  // sessions 已按 last_active_at 倒序，分区后仍保留各 agent 内最近活跃顺序。
  const visibleSessions = filterSessionsByRecentDays(sessions, recentDays);
  const sessionsByCli = new Map<CliType, SessionData[]>();
  for (const cliType of CLI_ORDER) {
    sessionsByCli.set(cliType, []);
  }
  for (const session of visibleSessions) {
    sessionsByCli.get(session.cliType)?.push(session);
  }

  const showHint =
    preferredTerminal === "system" && launchMode === "new-tab";

  return (
    <main className="app-shell">
      <header className="app-header">
        <div className="brand">
          <span className="app-icon">
            <Icon.Sparkle />
          </span>
          <div className="app-titles">
            <h1>Session Launcher</h1>
            <p className="app-subtitle">
              聚合 codex · claude-code · cursor，一键恢复工作现场
            </p>
          </div>
        </div>
        <button
          type="button"
          className="icon-btn"
          data-spin={refreshing}
          disabled={refreshing}
          onClick={() => void refreshSessions()}
          aria-label="刷新"
          title="刷新 session"
        >
          <Icon.Refresh />
        </button>
      </header>

      <div className="control-bar">
        <RecentDaysMenu
          value={recentDays}
          onChange={setRecentDays}
          visibleCount={visibleSessions.length}
          totalCount={sessions.length}
        />
        <LaunchSegmented value={launchMode} onChange={handleLaunchModeChange} />
        <TerminalMenu
          value={preferredTerminal}
          available={availableTerminals}
          onChange={handleTerminalChange}
        />
        <ThemeMenu value={themeMode} onChange={handleThemeModeChange} />
      </div>

      <div className="status-line">
        {statusMessage && (
          <span className="status-pill" data-type={statusType} data-pulse={loading}>
            <span className="status-dot" />
            {loading ? "正在扫描 session…" : statusMessage}
          </span>
        )}
      </div>

      {showHint && (
        <div className="status-line" style={{ marginTop: 0 }}>
          <span className="status-pill" data-type="info">
            Terminal.app 不支持新标签页，将打开新窗口
          </span>
        </div>
      )}

      {scanErrors.length > 0 && (
        <div className="scan-errors" aria-label="扫描失败的 CLI">
          {scanErrors.map((error) => (
            <span key={error.cliType} className="scan-error-item">
              {CLI_LABELS[error.cliType]}：{error.message}
            </span>
          ))}
        </div>
      )}

      {loading ? (
        <Skeleton />
      ) : (
        <div className="session-list">
          {CLI_ORDER.map((cliType) => (
            <AgentGroup
              key={cliType}
              cliType={cliType}
              sessions={sessionsByCli.get(cliType) ?? []}
              launchingId={launchingId}
              onLaunch={handleLaunch}
            />
          ))}
        </div>
      )}
    </main>
  );
}

export default App;
