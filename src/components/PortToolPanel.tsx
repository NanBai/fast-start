import type { RefObject } from "react";
import {
  AutoRefreshToggle,
  PortScopeSegmented,
  ProtocolMenu,
  SearchBox,
  ThemeMenu,
} from "./Controls";
import { Icon } from "./icons/Icon";
import { PortWorkspace } from "./PortWorkspace";
import { Skeleton } from "./Skeleton";
import type {
  PortProtocol,
  PortScope,
  PortUsage,
  ThemeMode,
} from "../types";

export type PortToolPanelProps = {
  ports: PortUsage[];
  visiblePorts: PortUsage[];
  scope: PortScope;
  protocol: PortProtocol | "all";
  searchQuery: string;
  loading: boolean;
  refreshing: boolean;
  terminatingIds: Set<string>;
  lastUpdated: string;
  diagnosticText: string;
  ignorePorts: number[];
  protectPorts: number[];
  projectPathPrefixes: string[];
  portAutoRefresh: boolean;
  themeMode: ThemeMode;
  searchInputRef: RefObject<HTMLInputElement | null>;
  onSearchChange: (value: string) => void;
  onScopeChange: (scope: PortScope) => void;
  onProtocolChange: (protocol: PortProtocol | "all") => void;
  onPortAutoRefreshChange: (enabled: boolean) => void | Promise<void>;
  onThemeModeChange: (mode: ThemeMode) => void | Promise<void>;
  onRefresh: () => void;
  onTerminate: (ports: PortUsage[]) => void;
  onNotify: (message: string, type: "info" | "success" | "error") => void;
  onIgnorePortsChange: (ports: number[]) => void;
  onProtectPortsChange: (ports: number[]) => void;
  onProjectPathPrefixesChange: (prefixes: string[]) => void;
};

/** Port 工具页：控制栏 + 列表工作区（从 App 壳拆出）。 */
export function PortToolPanel({
  ports,
  visiblePorts,
  scope,
  protocol,
  searchQuery,
  loading,
  refreshing,
  terminatingIds,
  lastUpdated,
  diagnosticText,
  ignorePorts,
  protectPorts,
  projectPathPrefixes,
  portAutoRefresh,
  themeMode,
  searchInputRef,
  onSearchChange,
  onScopeChange,
  onProtocolChange,
  onPortAutoRefreshChange,
  onThemeModeChange,
  onRefresh,
  onTerminate,
  onNotify,
  onIgnorePortsChange,
  onProtectPortsChange,
  onProjectPathPrefixesChange,
}: PortToolPanelProps) {
  return (
    <>
      <div className="control-bar">
        <div className="control-bar-main">
          <SearchBox
            value={searchQuery}
            onChange={onSearchChange}
            inputRef={searchInputRef}
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
                <PortScopeSegmented value={scope} onChange={onScopeChange} />
                <ProtocolMenu value={protocol} onChange={onProtocolChange} />
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
                  onChange={async (enabled) => {
                    await onPortAutoRefreshChange(enabled);
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

      <div className="workspace-panel">
        {loading ? (
          <Skeleton />
        ) : (
          <PortWorkspace
            ports={ports}
            visiblePorts={visiblePorts}
            scope={scope}
            loading={loading}
            refreshing={refreshing}
            terminatingIds={terminatingIds}
            lastUpdated={lastUpdated}
            diagnosticText={diagnosticText}
            ignorePorts={ignorePorts}
            protectPorts={protectPorts}
            projectPathPrefixes={projectPathPrefixes}
            onRefresh={onRefresh}
            onTerminate={onTerminate}
            onNotify={onNotify}
            onIgnorePortsChange={onIgnorePortsChange}
            onProtectPortsChange={onProtectPortsChange}
            onProjectPathPrefixesChange={onProjectPathPrefixesChange}
          />
        )}
      </div>
    </>
  );
}
