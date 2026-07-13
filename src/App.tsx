import { useEffect, useRef, useState } from "react";
import type { KeyboardEvent as ReactKeyboardEvent, MouseEvent } from "react";
import { AgentGroup } from "./components/AgentGroup";
import { ConfirmDialog } from "./components/ConfirmDialog";
import {
  AutoRefreshToggle,
  LaunchSegmented,
  PortScopeSegmented,
  ProtocolMenu,
  RecentDaysMenu,
  SearchBox,
  SessionListModeSegmented,
  TerminalMenu,
  ThemeMenu,
} from "./components/Controls";
import { ProjectBucket } from "./components/ProjectBucket";
import { Icon } from "./components/icons/Icon";
import { PortConfirmDialog } from "./components/PortConfirmDialog";
import { PortWorkspace } from "./components/PortWorkspace";
import { ProvidersWorkspace } from "./components/ProvidersWorkspace";
import { SessionContextMenu } from "./components/SessionContextMenu";
import { Skeleton } from "./components/Skeleton";
import { useGrokProviders } from "./hooks/useGrokProviders";
import { usePorts } from "./hooks/usePorts";
import { usePreferences } from "./hooks/usePreferences";
import { useSessions } from "./hooks/useSessions";
import { filterPorts } from "./lib/portUtils";
import {
  aggregateDiskUsage,
  filterSessionsByHealth,
  filterSessionsForQuickAccess,
  formatBytes,
  groupSessionsByProject,
  RecentDaysFilter,
  sanitizeFavoriteProjectDirs,
  sanitizeFavoriteSessionIds,
  sessionHealthBadge,
} from "./lib/sessionUtils";
import {
  APP_TOOL_LABELS,
  AppTool,
  CLI_LABELS,
  CLI_ORDER,
  CliType,
  PortProtocol,
  PortScope,
  SessionData,
  SessionHealthFilter,
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
  const [activeTool, setActiveTool] = useState<AppTool>("sessions");
  const [recentDays, setRecentDays] = useState<RecentDaysFilter>("7");
  const [healthFilter, setHealthFilter] = useState<SessionHealthFilter>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [portSearchQuery, setPortSearchQuery] = useState("");
  const [portScope, setPortScope] = useState<PortScope>("project");
  const [portProtocol, setPortProtocol] = useState<PortProtocol | "all">("all");
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const portSearchInputRef = useRef<HTMLInputElement>(null);
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
    favoriteSessionIds,
    portAutoRefresh,
    portIgnorePorts,
    portProtectPorts,
    portProjectPathPrefixes,
    sessionListMode,
    loadPreferences,
    handleTerminalChange,
    handleLaunchModeChange,
    handleThemeModeChange,
    handleFavoriteProjectDirsChange,
    handleFavoriteSessionIdsChange,
    handlePortAutoRefreshChange,
    handlePortIgnorePortsChange,
    handlePortProtectPortsChange,
    handlePortProjectPathPrefixesChange,
    handleSessionListModeChange,
  } = usePreferences(notifyStatus);

  const [diskUsageOpen, setDiskUsageOpen] = useState(false);

  const {
    sessions,
    scanErrors,
    loading,
    refreshing,
    launchingId,
    deletingId,
    pendingDelete,
    recentLaunches,
    commandPreview,
    healthById,
    selectedIds,
    pendingBulkDelete,
    bulkDeleting,
    loadSessions,
    refreshSessions,
    launchSession,
    previewLaunchCommand,
    clearCommandPreview,
    requestDeleteSession: requestDelete,
    cancelDeleteSession,
    confirmDeleteSession,
    toggleSessionSelected,
    clearSessionSelection,
    requestBulkDelete,
    cancelBulkDelete,
    confirmBulkDelete,
  } = useSessions(notifyStatus);

  const {
    ports,
    loading: portLoading,
    refreshing: portRefreshing,
    terminatingIds,
    pendingTerminate,
    lastScan,
    loadPorts,
    refreshPorts,
    requestTerminatePorts,
    cancelTerminatePorts,
    confirmTerminatePorts,
  } = usePorts(notifyStatus);

  const {
    profiles: grokProfiles,
    status: grokStatus,
    backups: grokBackups,
    health: grokHealth,
    layout: grokLayout,
    loading: grokLoading,
    busyId: grokBusyId,
    refreshAll: refreshGrokProviders,
    activate: activateGrokProfile,
    activateOfficial: activateGrokOfficial,
    applyPrivacy: applyGrokPrivacy,
    saveLayout: saveGrokLayout,
    importCurrent: importGrokCurrent,
    saveProfile: saveGrokProfile,
    removeProfile: removeGrokProfile,
    restoreBackup: restoreGrokBackup,
    fetchModels: fetchGrokModels,
    testConnection: testGrokConnection,
    previewApply: previewGrokApply,
  } = useGrokProviders(notifyStatus);

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

  function toggleFavoriteSession(sessionId: string) {
    const current = new Set(sanitizeFavoriteSessionIds(favoriteSessionIds, sessions));
    if (current.has(sessionId)) {
      current.delete(sessionId);
    } else {
      current.add(sessionId);
    }
    const next = sanitizeFavoriteSessionIds(Array.from(current), sessions);
    void handleFavoriteSessionIdsChange(next);
  }

  useEffect(() => {
    // 偏好与 session 扫描互不依赖：并行启动，缩短首屏等待。
    void (async () => {
      const prefs = loadPreferences().catch((error) => {
        notifyStatus(`偏好加载失败：${String(error)}`, "error");
      });
      const sessions = loadSessions();
      await Promise.all([prefs, sessions]);
    })();
  }, []);

  useEffect(() => {
    applyThemeMode(themeMode);
  }, [themeMode]);

  useEffect(() => {
    function handleGlobalShortcuts(event: globalThis.KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        if (activeTool === "providers") {
          return;
        }
        const input = activeTool === "ports" ? portSearchInputRef.current : searchInputRef.current;
        input?.focus();
        input?.select();
        return;
      }
      if (
        activeTool === "ports" &&
        (event.metaKey || event.ctrlKey) &&
        event.key.toLowerCase() === "r"
      ) {
        event.preventDefault();
        void refreshPorts();
      }
    }
    window.addEventListener("keydown", handleGlobalShortcuts);
    return () => window.removeEventListener("keydown", handleGlobalShortcuts);
  }, [activeTool, refreshPorts]);

  useEffect(() => {
    if (activeTool !== "ports" || lastScan) {
      return;
    }
    void loadPorts();
  }, [activeTool, lastScan]);

  useEffect(() => {
    if (activeTool !== "providers") {
      return;
    }
    void refreshGrokProviders(false);
  }, [activeTool]);

  useEffect(() => {
    if (activeTool !== "ports" || !portAutoRefresh) {
      return;
    }
    const timer = window.setInterval(() => void refreshPorts(false), 3000);
    return () => window.clearInterval(timer);
  }, [activeTool, portAutoRefresh, refreshPorts]);

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
  const favoriteSessionIdSet = new Set(
    sanitizeFavoriteSessionIds(favoriteSessionIds, sessions),
  );
  const scanErrorByCli = new Map(
    scanErrors.map((error) => [error.cliType, error.message] as const),
  );
  const quickAccess = filterSessionsForQuickAccess(sessions, {
    recentDays,
    query: searchQuery,
    favoriteProjectDirs: favoriteProjectDirSet,
    favoriteSessionIds: favoriteSessionIdSet,
    activeSessionId,
  });
  const visibleSessions = filterSessionsByHealth(
    quickAccess.sessions,
    healthById,
    healthFilter,
  );
  const hasSearchQuery = searchQuery.trim().length > 0;
  const activeQuickSessionId = hasSearchQuery ? quickAccess.activeSessionId : null;
  const sessionsByCli = new Map<CliType, SessionData[]>();
  for (const cliType of CLI_ORDER) {
    sessionsByCli.set(cliType, []);
  }
  for (const session of visibleSessions) {
    sessionsByCli.get(session.cliType)?.push(session);
  }
  const projectGroups =
    sessionListMode === "by-project"
      ? groupSessionsByProject(visibleSessions)
      : [];

  const showHint =
    preferredTerminal === "system" && launchMode === "new-tab";
  const visiblePorts = filterPorts(ports, {
    scope: portScope,
    protocol: portProtocol,
    query: portSearchQuery,
  });
  const portLastUpdated = lastScan
    ? new Date(lastScan.scannedAt).toLocaleTimeString()
    : "尚未刷新";
  const portDiagnostic = lastScan
    ? `raw ${lastScan.rawLineCount} lines, parsed ${lastScan.ports.length} ports`
    : "尚未执行扫描";

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
            <p className="app-subtitle" key={activeTool}>
              {activeTool === "ports"
                ? "监控本机开发端口，一键关闭残留服务"
                : activeTool === "providers"
                  ? "管理 Grok 上游供应商，一键切换 config.toml"
                  : "聚合 codex · claude-code · cursor · grok-build · opencode，一键恢复工作现场"}
            </p>
          </div>
        </div>
        <div
          className="tool-switcher"
          role="tablist"
          aria-label="工具切换"
          data-tool={activeTool}
        >
          {(["sessions", "ports", "providers"] as AppTool[]).map((tool) => (
            <button
              key={tool}
              type="button"
              role="tab"
              aria-selected={activeTool === tool}
              className="tool-tab"
              data-active={activeTool === tool}
              onClick={() => setActiveTool(tool)}
            >
              {tool === "sessions" ? (
                <Icon.Sessions />
              ) : tool === "ports" ? (
                <Icon.Port />
              ) : (
                <Icon.Grok />
              )}
              {APP_TOOL_LABELS[tool]}
            </button>
          ))}
        </div>
        <div className="header-actions">
          <button
            type="button"
            className="icon-btn"
            data-spin={
              activeTool === "ports"
                ? portRefreshing
                : activeTool === "providers"
                  ? grokLoading
                  : refreshing
            }
            disabled={
              activeTool === "ports"
                ? portRefreshing
                : activeTool === "providers"
                  ? grokLoading
                  : refreshing
            }
            onClick={() =>
              void (activeTool === "ports"
                ? refreshPorts()
                : activeTool === "providers"
                  ? refreshGrokProviders()
                  : refreshSessions())
            }
            aria-label="刷新"
            title={
              activeTool === "ports"
                ? "刷新端口"
                : activeTool === "providers"
                  ? "刷新供应商"
                  : "刷新 session"
            }
          >
            <Icon.Refresh />
          </button>
        </div>
      </header>

      <div className="control-bar">
        <div className="control-bar-main" key={activeTool}>
          {activeTool === "ports" ? (
            <>
              <SearchBox
                value={portSearchQuery}
                onChange={setPortSearchQuery}
                inputRef={portSearchInputRef}
                placeholder="搜索端口、进程、PID 或路径"
                ariaLabel="搜索端口"
              />
              <div className="control-groups">
                <section className="control-group" aria-label="筛选">
                  <div className="control-group-label">
                    <Icon.Filter />
                    筛选
                  </div>
                  <div className="control-group-body">
                    <PortScopeSegmented value={portScope} onChange={setPortScope} />
                    <ProtocolMenu value={portProtocol} onChange={setPortProtocol} />
                  </div>
                </section>
                <section className="control-group" aria-label="刷新">
                  <div className="control-group-label">
                    <Icon.Refresh />
                    刷新
                  </div>
                  <div className="control-group-body">
                    <AutoRefreshToggle
                      enabled={portAutoRefresh}
                      onChange={handlePortAutoRefreshChange}
                    />
                  </div>
                </section>
                <section className="control-group" aria-label="外观">
                  <div className="control-group-label">
                    <Icon.Appearance />
                    外观
                  </div>
                  <div className="control-group-body">
                    <ThemeMenu value={themeMode} onChange={handleThemeModeChange} />
                  </div>
                </section>
              </div>
            </>
          ) : activeTool === "providers" ? (
            <div className="control-groups control-groups-providers">
              <section className="control-group control-group-hint" aria-label="说明">
                <div className="control-group-label">说明</div>
                <div className="control-group-body">
                  <p className="providers-control-hint muted">
                    切换后<strong>新开</strong> Grok 会话才会读取新 config；不会结束已运行的会话。
                  </p>
                </div>
              </section>
              <section className="control-group" aria-label="外观">
                <div className="control-group-label">
                  <Icon.Appearance />
                  外观
                </div>
                <div className="control-group-body">
                  <ThemeMenu value={themeMode} onChange={handleThemeModeChange} />
                </div>
              </section>
            </div>
          ) : (
            <>
              <SearchBox
                value={searchQuery}
                onChange={handleSearchQueryChange}
                inputRef={searchInputRef}
                onKeyDown={handleSearchKeyDown}
              />
              <div className="control-groups">
                <section className="control-group" aria-label="筛选">
                  <div className="control-group-label">
                    <Icon.Filter />
                    筛选
                  </div>
                  <div className="control-group-body">
                    <RecentDaysMenu
                      value={recentDays}
                      onChange={handleRecentDaysChange}
                      visibleCount={visibleSessions.length}
                      totalCount={sessions.length}
                    />
                    <label className="health-filter">
                      <span className="sr-only">健康筛选</span>
                      <select
                        value={healthFilter}
                        onChange={(e) =>
                          setHealthFilter(e.target.value as SessionHealthFilter)
                        }
                        aria-label="按健康状态筛选"
                      >
                        <option value="all">全部状态</option>
                        <option value="stale">陈旧（缺目录/源）</option>
                        <option value="missing_cwd">缺工作目录</option>
                        <option value="missing_source">缺会话源</option>
                      </select>
                    </label>
                    <SessionListModeSegmented
                      value={sessionListMode}
                      onChange={handleSessionListModeChange}
                    />
                  </div>
                </section>
                <section className="control-group" aria-label="启动">
                  <div className="control-group-label">
                    <Icon.Launch />
                    启动
                  </div>
                  <div className="control-group-body">
                    <LaunchSegmented value={launchMode} onChange={handleLaunchModeChange} />
                    <TerminalMenu
                      value={preferredTerminal}
                      available={availableTerminals}
                      onChange={handleTerminalChange}
                    />
                  </div>
                </section>
                <section className="control-group" aria-label="外观">
                  <div className="control-group-label">
                    <Icon.Appearance />
                    外观
                  </div>
                  <div className="control-group-body">
                    <ThemeMenu value={themeMode} onChange={handleThemeModeChange} />
                  </div>
                </section>
              </div>
            </>
          )}
        </div>
      </div>

      <div className="status-line">
        {statusMessage && (
          <span
            key={`${statusType}:${statusMessage}`}
            className="status-pill"
            data-type={statusType}
            data-pulse={
              activeTool === "ports"
                ? portLoading
                : activeTool === "providers"
                  ? grokLoading
                  : loading
            }
          >
            <span className="status-dot" />
            {activeTool === "ports" && portLoading
              ? "正在扫描端口…"
              : activeTool === "providers" && grokLoading
                ? "正在加载供应商…"
                : loading
                  ? "正在扫描 session…"
                  : statusMessage}
          </span>
        )}
      </div>

      {activeTool === "sessions" && showHint && (
        <div className="status-line" style={{ marginTop: 0 }}>
          <span className="status-pill" data-type="info">
            Terminal.app 不支持新标签页，将打开新窗口
          </span>
        </div>
      )}

      {activeTool === "sessions" && recentLaunches.length > 0 && (
        <div className="recent-launches" aria-label="最近启动">
          <span className="recent-launches-label">最近启动</span>
          <div className="recent-launches-list">
            {recentLaunches.slice(0, 8).map((item) => {
              const stillExists = sessions.some(
                (session) => session.id === item.sessionListId,
              );
              const busyChip =
                launchingId === item.sessionListId ||
                deletingId === item.sessionListId;
              return (
                <button
                  key={`${item.sessionListId}-${item.launchedAt}`}
                  type="button"
                  className="recent-launch-chip"
                  data-stale={!stillExists}
                  disabled={busyChip || !stillExists}
                  title={
                    stillExists
                      ? `${item.projectDir}\n右键预览命令`
                      : "该 session 已不存在，刷新后将清理"
                  }
                  onClick={() => {
                    if (!stillExists) return;
                    void handleLaunch(item.sessionListId);
                  }}
                  onContextMenu={(event) => {
                    event.preventDefault();
                    if (!stillExists) return;
                    void previewLaunchCommand(item.sessionListId);
                  }}
                >
                  <span className="recent-launch-cli">{CLI_LABELS[item.cliType]}</span>
                  <span className="recent-launch-name">
                    {item.summary?.trim() || item.projectName}
                  </span>
                </button>
              );
            })}
          </div>
        </div>
      )}

      {activeTool === "sessions" && selectedIds.size > 0 && (
        <div className="bulk-select-bar" aria-label="批量选择">
          <span>已选 {selectedIds.size} 条</span>
          <button type="button" className="btn" onClick={clearSessionSelection}>
            取消选择
          </button>
          <button
            type="button"
            className="btn danger"
            disabled={bulkDeleting}
            onClick={requestBulkDelete}
          >
            批量删除
          </button>
        </div>
      )}

      {activeTool === "sessions" && !loading && sessions.length > 0 && (
        <div className="disk-usage-panel" aria-label="磁盘占用">
          <button
            type="button"
            className="disk-usage-toggle"
            onClick={() => setDiskUsageOpen((open) => !open)}
            aria-expanded={diskUsageOpen}
          >
            磁盘占用（源载体近似）{diskUsageOpen ? " ▾" : " ▸"}
          </button>
          {diskUsageOpen && (
            <div className="disk-usage-body">
              <p className="muted">
                基于已探测的 session 源体积；OpenCode 行为未知（不按整库计）。最多统计当前列表前 200 条。
              </p>
              <div className="disk-usage-columns">
                <div>
                  <h4>按 CLI</h4>
                  <ul>
                    {aggregateDiskUsage(sessions, healthById, "cli").map((b) => (
                      <li key={b.key}>
                        <span>{b.label}</span>
                        <span>
                          {formatBytes(b.bytes)}
                          {b.unknownCount > 0 ? ` · ${b.unknownCount} 未知` : ""}
                          {b.sizeCapped ? " · 有截断" : ""}
                        </span>
                      </li>
                    ))}
                  </ul>
                </div>
                <div>
                  <h4>按项目</h4>
                  <ul>
                    {aggregateDiskUsage(sessions, healthById, "project")
                      .slice(0, 12)
                      .map((b) => (
                        <li key={b.key} title={b.key}>
                          <span>{b.label}</span>
                          <span>
                            {formatBytes(b.bytes)}
                            {b.unknownCount > 0 ? ` · ${b.unknownCount} 未知` : ""}
                          </span>
                        </li>
                      ))}
                  </ul>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {activeTool === "sessions" && commandPreview && (
        <div className="command-preview" aria-label="命令预览">
          <code>
            {commandPreview.cd ? `cd ${commandPreview.cwd} && ` : ""}
            {commandPreview.program} {commandPreview.args.join(" ")}
          </code>
          <button type="button" className="btn" onClick={clearCommandPreview}>
            关闭预览
          </button>
        </div>
      )}

      {activeTool === "sessions" && scanErrors.length > 0 && (
        <div className="scan-errors" aria-label="扫描失败的 CLI">
          {scanErrors.map((error) => (
            <span key={error.cliType} className="scan-error-item">
              {CLI_LABELS[error.cliType]}：{error.message}
            </span>
          ))}
        </div>
      )}

      <div className="workspace-panel" key={activeTool}>
      {activeTool === "ports" ? (
        portLoading ? (
          <Skeleton />
        ) : (
          <PortWorkspace
            ports={ports}
            visiblePorts={visiblePorts}
            scope={portScope}
            loading={portLoading}
            refreshing={portRefreshing}
            terminatingIds={terminatingIds}
            lastUpdated={portLastUpdated}
            diagnosticText={portDiagnostic}
            ignorePorts={portIgnorePorts}
            protectPorts={portProtectPorts}
            projectPathPrefixes={portProjectPathPrefixes}
            onRefresh={() => void refreshPorts()}
            onTerminate={requestTerminatePorts}
            onNotify={notifyStatus}
            onIgnorePortsChange={(ports) =>
              void handlePortIgnorePortsChange(ports).then(() => refreshPorts())
            }
            onProtectPortsChange={(ports) => void handlePortProtectPortsChange(ports)}
            onProjectPathPrefixesChange={(prefixes) =>
              void handlePortProjectPathPrefixesChange(prefixes).then(() =>
                refreshPorts(),
              )
            }
          />
        )
      ) : activeTool === "providers" ? (
        grokLoading && grokStatus == null ? (
          <Skeleton />
        ) : (
          <ProvidersWorkspace
            profiles={grokProfiles}
            status={grokStatus}
            backups={grokBackups}
            health={grokHealth}
            layout={grokLayout}
            loading={grokLoading}
            busyId={grokBusyId}
            onRefresh={() => void refreshGrokProviders()}
            onActivate={(id) => void activateGrokProfile(id)}
            onActivateOfficial={() => void activateGrokOfficial()}
            onApplyPrivacy={() => void applyGrokPrivacy()}
            onSaveLayout={(next) => saveGrokLayout(next)}
            onImport={() => void importGrokCurrent()}
            onSave={(profile, activateAfter) => saveGrokProfile(profile, activateAfter)}
            onDelete={(id) => void removeGrokProfile(id)}
            onRestore={(file) => void restoreGrokBackup(file)}
            onFetchModels={fetchGrokModels}
            onTestConnection={testGrokConnection}
            onPreviewApply={previewGrokApply}
          />
        )
      ) : loading ? (
        <Skeleton />
      ) : hasSearchQuery && quickAccess.matchCount === 0 ? (
        <p className="state-line">没有匹配的 session</p>
      ) : (
        <div className="session-list" data-list-mode={sessionListMode} key={sessionListMode}>
          {sessionListMode === "by-project" ? (
            projectGroups.length === 0 ? (
              <p className="state-line">暂无 session</p>
            ) : (
              projectGroups.map((group) => (
                <ProjectBucket
                  key={group.projectDir}
                  projectDir={group.projectDir}
                  projectName={group.projectName}
                  sessions={group.sessions}
                  favorite={favoriteProjectDirSet.has(group.projectDir)}
                  forceOpen={hasSearchQuery}
                  showCliLabel
                  favoriteSessionIds={favoriteSessionIdSet}
                  activeSessionId={activeQuickSessionId}
                  launchingId={launchingId}
                  deletingId={deletingId}
                  healthBadgeFor={(id) => sessionHealthBadge(healthById.get(id))}
                  selectedIds={selectedIds}
                  onToggleSelected={toggleSessionSelected}
                  onLaunch={handleLaunch}
                  onToggleFavorite={toggleFavoriteProject}
                  onToggleSessionFavorite={toggleFavoriteSession}
                  onSessionContextMenu={handleSessionContextMenu}
                />
              ))
            )
          ) : (
            CLI_ORDER.map((cliType) => (
              <AgentGroup
                key={cliType}
                cliType={cliType}
                sessions={sessionsByCli.get(cliType) ?? []}
                favoriteProjectDirs={favoriteProjectDirSet}
                favoriteSessionIds={favoriteSessionIdSet}
                forceOpen={hasSearchQuery}
                scanError={scanErrorByCli.get(cliType) ?? null}
                activeSessionId={activeQuickSessionId}
                launchingId={launchingId}
                deletingId={deletingId}
                healthBadgeFor={(id) => sessionHealthBadge(healthById.get(id))}
                selectedIds={selectedIds}
                onToggleSelected={toggleSessionSelected}
                onLaunch={handleLaunch}
                onToggleFavoriteProject={toggleFavoriteProject}
                onToggleSessionFavorite={toggleFavoriteSession}
                onSessionContextMenu={handleSessionContextMenu}
              />
            ))
          )}
        </div>
      )}
      </div>

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

      {pendingBulkDelete && (
        <div className="dialog-backdrop" role="presentation">
          <section
            className="confirm-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="bulk-delete-title"
          >
            <div className="confirm-copy">
              <h2 id="bulk-delete-title">批量删除 session？</h2>
              <p>
                将删除已选中的 <strong>{selectedIds.size}</strong>{" "}
                条（不可撤销，单次上限 50）。部分失败时会保留失败项并展示原因。
              </p>
            </div>
            <div className="confirm-actions">
              <button
                type="button"
                className="confirm-btn"
                disabled={bulkDeleting}
                onClick={cancelBulkDelete}
              >
                取消
              </button>
              <button
                type="button"
                className="confirm-btn danger"
                disabled={bulkDeleting}
                onClick={() => void confirmBulkDelete()}
              >
                {bulkDeleting ? "删除中" : `删除 ${selectedIds.size} 条`}
              </button>
            </div>
          </section>
        </div>
      )}

      {pendingTerminate && (
        <PortConfirmDialog
          ports={pendingTerminate}
          closing={pendingTerminate.some((port) => terminatingIds.has(port.id))}
          onCancel={cancelTerminatePorts}
          onConfirm={() => void confirmTerminatePorts()}
        />
      )}
    </main>
  );
}

export default App;
