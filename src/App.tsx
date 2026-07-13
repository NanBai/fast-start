import { useEffect, useRef, useState } from "react";
import { Icon } from "./components/icons/Icon";
import { PortConfirmDialog } from "./components/PortConfirmDialog";
import { PortToolPanel } from "./components/PortToolPanel";
import { ProvidersToolPanel } from "./components/ProvidersToolPanel";
import { SessionWorkspace } from "./components/SessionWorkspace";
import { useGrokProviders } from "./hooks/useGrokProviders";
import { usePorts } from "./hooks/usePorts";
import { usePreferences } from "./hooks/usePreferences";
import { useSessions } from "./hooks/useSessions";
import { filterPorts } from "./lib/portUtils";
import {
  APP_TOOL_LABELS,
  AppTool,
  PortProtocol,
  PortScope,
  StatusType,
  ThemeMode,
} from "./types";
import "./App.css";

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
  const [portSearchQuery, setPortSearchQuery] = useState("");
  const [portScope, setPortScope] = useState<PortScope>("project");
  const [portProtocol, setPortProtocol] = useState<PortProtocol | "all">("all");
  const searchInputRef = useRef<HTMLInputElement>(null);
  const portSearchInputRef = useRef<HTMLInputElement>(null);
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
    selectSessionIds,
    requestBulkDelete,
    cancelBulkDelete,
    confirmBulkDelete,
    inspectHealthForSessions,
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

  useEffect(() => {
    void (async () => {
      const prefs = loadPreferences().catch((error) => {
        notifyStatus(`偏好加载失败：${String(error)}`, "error");
      });
      const sessionsLoad = loadSessions();
      await Promise.all([prefs, sessionsLoad]);
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
        const input =
          activeTool === "ports" ? portSearchInputRef.current : searchInputRef.current;
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

      {activeTool === "sessions" && (
        <SessionWorkspace
          sessions={sessions}
          scanErrors={scanErrors}
          loading={loading}
          launchingId={launchingId}
          deletingId={deletingId}
          pendingDelete={pendingDelete}
          recentLaunches={recentLaunches}
          commandPreview={commandPreview}
          healthById={healthById}
          selectedIds={selectedIds}
          pendingBulkDelete={pendingBulkDelete}
          bulkDeleting={bulkDeleting}
          favoriteProjectDirs={favoriteProjectDirs}
          favoriteSessionIds={favoriteSessionIds}
          sessionListMode={sessionListMode}
          preferredTerminal={preferredTerminal}
          availableTerminals={availableTerminals}
          launchMode={launchMode}
          themeMode={themeMode}
          searchInputRef={searchInputRef}
          notifyStatus={notifyStatus}
          onLaunchModeChange={handleLaunchModeChange}
          onTerminalChange={handleTerminalChange}
          onThemeModeChange={handleThemeModeChange}
          onSessionListModeChange={handleSessionListModeChange}
          onFavoriteProjectDirsChange={handleFavoriteProjectDirsChange}
          onFavoriteSessionIdsChange={handleFavoriteSessionIdsChange}
          launchSession={launchSession}
          previewLaunchCommand={previewLaunchCommand}
          clearCommandPreview={clearCommandPreview}
          requestDeleteSession={requestDelete}
          cancelDeleteSession={cancelDeleteSession}
          confirmDeleteSession={confirmDeleteSession}
          toggleSessionSelected={toggleSessionSelected}
          clearSessionSelection={clearSessionSelection}
          selectSessionIds={selectSessionIds}
          requestBulkDelete={requestBulkDelete}
          cancelBulkDelete={cancelBulkDelete}
          confirmBulkDelete={confirmBulkDelete}
          inspectHealthForSessions={inspectHealthForSessions}
        />
      )}

      {activeTool === "ports" && (
        <PortToolPanel
          ports={ports}
          visiblePorts={visiblePorts}
          scope={portScope}
          protocol={portProtocol}
          searchQuery={portSearchQuery}
          loading={portLoading}
          refreshing={portRefreshing}
          terminatingIds={terminatingIds}
          lastUpdated={portLastUpdated}
          diagnosticText={portDiagnostic}
          ignorePorts={portIgnorePorts}
          protectPorts={portProtectPorts}
          projectPathPrefixes={portProjectPathPrefixes}
          portAutoRefresh={portAutoRefresh}
          themeMode={themeMode}
          searchInputRef={portSearchInputRef}
          onSearchChange={setPortSearchQuery}
          onScopeChange={setPortScope}
          onProtocolChange={setPortProtocol}
          onPortAutoRefreshChange={handlePortAutoRefreshChange}
          onThemeModeChange={handleThemeModeChange}
          onRefresh={() => void refreshPorts()}
          onTerminate={requestTerminatePorts}
          onNotify={notifyStatus}
          onIgnorePortsChange={(next) =>
            void handlePortIgnorePortsChange(next).then(() => refreshPorts())
          }
          onProtectPortsChange={(next) => void handlePortProtectPortsChange(next)}
          onProjectPathPrefixesChange={(prefixes) =>
            void handlePortProjectPathPrefixesChange(prefixes).then(() => refreshPorts())
          }
        />
      )}

      {activeTool === "providers" && (
        <ProvidersToolPanel
          profiles={grokProfiles}
          status={grokStatus}
          backups={grokBackups}
          health={grokHealth}
          layout={grokLayout}
          loading={grokLoading}
          busyId={grokBusyId}
          themeMode={themeMode}
          onThemeModeChange={handleThemeModeChange}
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
