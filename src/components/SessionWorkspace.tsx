import { useEffect, useState } from "react";
import type { KeyboardEvent as ReactKeyboardEvent, MouseEvent, RefObject } from "react";
import { AgentGroup } from "./AgentGroup";
import { ConfirmDialog } from "./ConfirmDialog";
import {
  LaunchSegmented,
  RecentDaysMenu,
  SearchBox,
  SessionListModeSegmented,
  TerminalMenu,
  ThemeMenu,
} from "./Controls";
import { ProjectBucket } from "./ProjectBucket";
import { Icon } from "./icons/Icon";
import { SessionContextMenu } from "./SessionContextMenu";
import { Skeleton } from "./Skeleton";
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
} from "../lib/sessionUtils";
import {
  CLI_LABELS,
  CLI_ORDER,
  CliType,
  LaunchCommandPreview,
  LaunchMode,
  RecentLaunch,
  SessionData,
  SessionHealth,
  SessionHealthFilter,
  SessionListMode,
  TerminalType,
  ThemeMode,
  CliScanError,
} from "../types";

type SessionMenuState = {
  session: SessionData;
  x: number;
  y: number;
};

export type SessionWorkspaceProps = {
  sessions: SessionData[];
  scanErrors: CliScanError[];
  loading: boolean;
  launchingId: string | null;
  deletingId: string | null;
  pendingDelete: SessionData | null;
  recentLaunches: RecentLaunch[];
  commandPreview: LaunchCommandPreview | null;
  healthById: Map<string, SessionHealth>;
  selectedIds: Set<string>;
  pendingBulkDelete: boolean;
  bulkDeleting: boolean;
  favoriteProjectDirs: string[];
  favoriteSessionIds: string[];
  sessionListMode: SessionListMode;
  preferredTerminal: TerminalType;
  availableTerminals: TerminalType[];
  launchMode: LaunchMode;
  themeMode: ThemeMode;
  searchInputRef: RefObject<HTMLInputElement | null>;
  notifyStatus: (message: string, type: "info" | "success" | "error") => void;
  onLaunchModeChange: (mode: LaunchMode) => void | Promise<void>;
  onTerminalChange: (terminal: TerminalType) => void | Promise<void>;
  onThemeModeChange: (mode: ThemeMode) => void | Promise<void>;
  onSessionListModeChange: (mode: SessionListMode) => void | Promise<void>;
  onFavoriteProjectDirsChange: (dirs: string[]) => void | Promise<void>;
  onFavoriteSessionIdsChange: (ids: string[]) => void | Promise<void>;
  launchSession: (sessionId: string) => Promise<void>;
  previewLaunchCommand: (sessionId: string) => Promise<unknown>;
  clearCommandPreview: () => void;
  requestDeleteSession: (session: SessionData) => void;
  cancelDeleteSession: () => void;
  confirmDeleteSession: () => Promise<void>;
  toggleSessionSelected: (sessionId: string) => void;
  clearSessionSelection: () => void;
  requestBulkDelete: () => void;
  cancelBulkDelete: () => void;
  confirmBulkDelete: () => Promise<void>;
  inspectHealthForSessions: (list: SessionData[]) => Promise<void>;
};

export function SessionWorkspace(props: SessionWorkspaceProps) {
  const {
    sessions,
    scanErrors,
    loading,
    launchingId,
    deletingId,
    pendingDelete,
    recentLaunches,
    commandPreview,
    healthById,
    selectedIds,
    pendingBulkDelete,
    bulkDeleting,
    favoriteProjectDirs,
    favoriteSessionIds,
    sessionListMode,
    preferredTerminal,
    availableTerminals,
    launchMode,
    themeMode,
    searchInputRef,
    notifyStatus,
    onLaunchModeChange,
    onTerminalChange,
    onThemeModeChange,
    onSessionListModeChange,
    onFavoriteProjectDirsChange,
    onFavoriteSessionIdsChange,
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
    inspectHealthForSessions,
  } = props;

  const [recentDays, setRecentDays] = useState<RecentDaysFilter>("7");
  const [healthFilter, setHealthFilter] = useState<SessionHealthFilter>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [sessionMenu, setSessionMenu] = useState<SessionMenuState | null>(null);
  const [diskUsageOpen, setDiskUsageOpen] = useState(false);

  // 按需健康探测：陈旧筛选或磁盘面板打开时再 inspect
  useEffect(() => {
    const needHealth = healthFilter !== "all" || diskUsageOpen;
    if (!needHealth || sessions.length === 0) {
      return;
    }
    void inspectHealthForSessions(sessions);
  }, [healthFilter, diskUsageOpen, sessions, inspectHealthForSessions]);

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
    void onFavoriteProjectDirsChange(next);
  }

  function toggleFavoriteSession(sessionId: string) {
    const current = new Set(sanitizeFavoriteSessionIds(favoriteSessionIds, sessions));
    if (current.has(sessionId)) {
      current.delete(sessionId);
    } else {
      current.add(sessionId);
    }
    const next = sanitizeFavoriteSessionIds(Array.from(current), sessions);
    void onFavoriteSessionIdsChange(next);
  }

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
    <>
      <div className="control-bar">
        <div className="control-bar-main">
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
                  onChange={onSessionListModeChange}
                />
              </div>
            </section>
            <section className="control-group" aria-label="启动">
              <div className="control-group-label">
                <Icon.Launch />
                启动
              </div>
              <div className="control-group-body">
                <LaunchSegmented
                  value={launchMode}
                  onChange={async (mode) => {
                    await onLaunchModeChange(mode);
                  }}
                />
                <TerminalMenu
                  value={preferredTerminal}
                  available={availableTerminals}
                  onChange={async (terminal) => {
                    await onTerminalChange(terminal);
                  }}
                />
              </div>
            </section>
            <section className="control-group" aria-label="外观">
              <div className="control-group-label">
                <Icon.Appearance />
                外观
              </div>
              <div className="control-group-body">
                <ThemeMenu
                  value={themeMode}
                  onChange={async (mode) => {
                    await onThemeModeChange(mode);
                  }}
                />
              </div>
            </section>
          </div>
        </div>
      </div>

      {showHint && (
        <div className="status-line" style={{ marginTop: 0 }}>
          <span className="status-pill" data-type="info">
            Terminal.app 不支持新标签页，将打开新窗口
          </span>
        </div>
      )}

      {/* 勾选后吸顶，滚动列表时仍可点批量删除 */}
      {selectedIds.size > 0 && (
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

      {recentLaunches.length > 0 && (
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

      {!loading && sessions.length > 0 && (
        <div className="disk-usage-panel" aria-label="磁盘占用">
          <button
            type="button"
            className="disk-usage-toggle"
            onClick={() => setDiskUsageOpen((open) => !open)}
            aria-expanded={diskUsageOpen}
          >
            磁盘占用{diskUsageOpen ? " ▾" : " ▸"}
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

      {commandPreview && (
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

      {scanErrors.length > 0 && (
        <div className="scan-errors" aria-label="扫描失败的 CLI">
          {scanErrors.map((error) => (
            <span key={error.cliType} className="scan-error-item">
              {CLI_LABELS[error.cliType]}：{error.message}
            </span>
          ))}
        </div>
      )}

      <div className="workspace-panel">
        {loading ? (
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
    </>
  );
}
