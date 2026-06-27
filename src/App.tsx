import { useEffect, useRef, useState } from "react";
import type { KeyboardEvent as ReactKeyboardEvent, MouseEvent } from "react";
import { AgentGroup } from "./components/AgentGroup";
import { ConfirmDialog } from "./components/ConfirmDialog";
import {
  LaunchSegmented,
  RecentDaysMenu,
  SearchBox,
  TerminalMenu,
  ThemeMenu,
} from "./components/Controls";
import { Icon } from "./components/icons/Icon";
import { SessionContextMenu } from "./components/SessionContextMenu";
import { Skeleton } from "./components/Skeleton";
import { usePreferences } from "./hooks/usePreferences";
import { useSessions } from "./hooks/useSessions";
import {
  filterSessionsForQuickAccess,
  RecentDaysFilter,
  sanitizeFavoriteProjectDirs,
} from "./lib/sessionUtils";
import {
  CLI_LABELS,
  CLI_ORDER,
  CliType,
  SessionData,
  StatusType,
  ThemeMode,
} from "./types";
import "./App.css";

type SessionMenuState = {
  session: SessionData;
  x: number;
  y: number;
};

function applyThemeMode(mode: ThemeMode) {
  const root = document.documentElement;
  if (mode === "system") {
    delete root.dataset.theme;
  } else {
    root.dataset.theme = mode;
  }
}

function App() {
  const [recentDays, setRecentDays] = useState<RecentDaysFilter>("7");
  const [searchQuery, setSearchQuery] = useState("");
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const [sessionMenu, setSessionMenu] = useState<SessionMenuState | null>(null);
  const [statusMessage, setStatusMessage] = useState<string>("");
  const [statusType, setStatusType] = useState<StatusType>("info");

  function notifyStatus(message: string, type: StatusType) {
    setStatusMessage(message);
    setStatusType(type);
  }

  const {
    availableTerminals,
    preferredTerminal,
    launchMode,
    themeMode,
    favoriteProjectDirs,
    loadPreferences,
    handleTerminalChange,
    handleLaunchModeChange,
    handleThemeModeChange,
    handleFavoriteProjectDirsChange,
  } = usePreferences(notifyStatus);

  const {
    sessions,
    scanErrors,
    loading,
    refreshing,
    launchingId,
    deletingId,
    pendingDelete,
    loadSessions,
    refreshSessions,
    launchSession,
    requestDeleteSession: requestDelete,
    cancelDeleteSession,
    confirmDeleteSession,
  } = useSessions(notifyStatus);

  async function handleLaunch(sessionId: string) {
    setSessionMenu(null);
    await launchSession(sessionId);
  }

  function handleSessionContextMenu(
    session: SessionData,
    event: MouseEvent<HTMLDivElement>,
  ) {
    event.preventDefault();
    if (launchingId === session.id || deletingId === session.id) {
      return;
    }
    setSessionMenu({
      session,
      x: event.clientX,
      y: event.clientY,
    });
  }

  function requestDeleteSession(session: SessionData) {
    setSessionMenu(null);
    requestDelete(session);
  }

  function handleSearchQueryChange(value: string) {
    setSearchQuery(value);
    setActiveSessionId(null);
  }

  function handleRecentDaysChange(value: RecentDaysFilter) {
    setRecentDays(value);
    setActiveSessionId(null);
  }

  function toggleFavoriteProject(projectDir: string) {
    const current = new Set(sanitizeFavoriteProjectDirs(favoriteProjectDirs, sessions));
    if (current.has(projectDir)) {
      current.delete(projectDir);
    } else {
      current.add(projectDir);
    }
    const next = sanitizeFavoriteProjectDirs(Array.from(current), sessions);
    void handleFavoriteProjectDirsChange(next);
  }

  useEffect(() => {
    void (async () => {
      try {
        await loadPreferences();
      } catch (error) {
        notifyStatus(`偏好加载失败：${String(error)}`, "error");
      }
      await loadSessions();
    })();
  }, []);

  useEffect(() => {
    applyThemeMode(themeMode);
  }, [themeMode]);

  useEffect(() => {
    function focusSearch(event: globalThis.KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        searchInputRef.current?.focus();
        searchInputRef.current?.select();
      }
    }
    window.addEventListener("keydown", focusSearch);
    return () => window.removeEventListener("keydown", focusSearch);
  }, []);

  useEffect(() => {
    if (!sessionMenu) {
      return;
    }
    function closeMenu() {
      setSessionMenu(null);
    }
    function closeMenuOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setSessionMenu(null);
      }
    }
    window.addEventListener("click", closeMenu);
    window.addEventListener("keydown", closeMenuOnEscape);
    return () => {
      window.removeEventListener("click", closeMenu);
      window.removeEventListener("keydown", closeMenuOnEscape);
    };
  }, [sessionMenu]);

  // 先按 agent 分区；每个 agent 内部再按工作目录聚合。
  // sessions 已按 last_active_at 倒序，分区后仍保留各 agent 内最近活跃顺序。
  const favoriteProjectDirSet = new Set(
    sanitizeFavoriteProjectDirs(favoriteProjectDirs, sessions),
  );
  const quickAccess = filterSessionsForQuickAccess(sessions, {
    recentDays,
    query: searchQuery,
    favoriteProjectDirs: favoriteProjectDirSet,
    activeSessionId,
  });
  const visibleSessions = quickAccess.sessions;
  const hasSearchQuery = searchQuery.trim().length > 0;
  const activeQuickSessionId = hasSearchQuery ? quickAccess.activeSessionId : null;
  const sessionsByCli = new Map<CliType, SessionData[]>();
  for (const cliType of CLI_ORDER) {
    sessionsByCli.set(cliType, []);
  }
  for (const session of visibleSessions) {
    sessionsByCli.get(session.cliType)?.push(session);
  }

  const showHint =
    preferredTerminal === "system" && launchMode === "new-tab";

  function moveActiveSession(delta: 1 | -1) {
    if (quickAccess.sessions.length === 0) {
      setActiveSessionId(null);
      return;
    }
    const currentIndex = quickAccess.sessions.findIndex(
      (session) => session.id === quickAccess.activeSessionId,
    );
    const safeIndex = currentIndex >= 0 ? currentIndex : 0;
    const nextIndex =
      (safeIndex + delta + quickAccess.sessions.length) %
      quickAccess.sessions.length;
    setActiveSessionId(quickAccess.sessions[nextIndex].id);
  }

  function handleSearchKeyDown(event: ReactKeyboardEvent<HTMLInputElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      if (searchQuery) {
        handleSearchQueryChange("");
      } else {
        searchInputRef.current?.blur();
      }
      return;
    }

    if (!hasSearchQuery) {
      return;
    }

    if (event.key === "ArrowDown") {
      event.preventDefault();
      moveActiveSession(1);
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      moveActiveSession(-1);
      return;
    }

    if (event.key === "Enter" && activeQuickSessionId) {
      event.preventDefault();
      if (launchingId === activeQuickSessionId || deletingId === activeQuickSessionId) {
        notifyStatus("当前 session 正在处理中", "info");
        return;
      }
      void handleLaunch(activeQuickSessionId);
    }
  }

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
        <SearchBox
          value={searchQuery}
          onChange={handleSearchQueryChange}
          inputRef={searchInputRef}
          onKeyDown={handleSearchKeyDown}
        />
        <RecentDaysMenu
          value={recentDays}
          onChange={handleRecentDaysChange}
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
      ) : hasSearchQuery && quickAccess.matchCount === 0 ? (
        <p className="state-line">没有匹配的 session</p>
      ) : (
        <div className="session-list">
          {CLI_ORDER.map((cliType) => (
            <AgentGroup
              key={cliType}
              cliType={cliType}
              sessions={sessionsByCli.get(cliType) ?? []}
              favoriteProjectDirs={favoriteProjectDirSet}
              forceOpen={hasSearchQuery}
              activeSessionId={activeQuickSessionId}
              launchingId={launchingId}
              deletingId={deletingId}
              onLaunch={handleLaunch}
              onToggleFavoriteProject={toggleFavoriteProject}
              onSessionContextMenu={handleSessionContextMenu}
            />
          ))}
        </div>
      )}

      {sessionMenu && (
        <SessionContextMenu
          session={sessionMenu.session}
          x={sessionMenu.x}
          y={sessionMenu.y}
          disabled={deletingId === sessionMenu.session.id}
          onDelete={requestDeleteSession}
        />
      )}

      {pendingDelete && (
        <ConfirmDialog
          session={pendingDelete}
          deleting={deletingId === pendingDelete.id}
          onCancel={cancelDeleteSession}
          onConfirm={() => void confirmDeleteSession()}
        />
      )}
    </main>
  );
}

export default App;
