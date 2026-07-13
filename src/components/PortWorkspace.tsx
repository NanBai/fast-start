import { useEffect, useState } from "react";
import { openPath, openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import { Icon } from "./icons/Icon";
import {
  groupPorts,
  groupPortsByWorkingDirectory,
  groupSummary,
  loopbackBrowserUrl,
  portMetrics,
  portProcessLabel,
  protocolLabel,
  serverURLLabel,
  shortPath,
} from "../lib/portUtils";
import { PortScope, PortUsage } from "../types";

export function PortWorkspace({
  ports,
  visiblePorts,
  scope,
  loading,
  refreshing,
  terminatingIds,
  lastUpdated,
  diagnosticText,
  ignorePorts,
  protectPorts,
  projectPathPrefixes,
  onRefresh,
  onTerminate,
  onNotify,
  onIgnorePortsChange,
  onProtectPortsChange,
  onProjectPathPrefixesChange,
}: {
  ports: PortUsage[];
  visiblePorts: PortUsage[];
  scope: PortScope;
  loading: boolean;
  refreshing: boolean;
  terminatingIds: Set<string>;
  lastUpdated: string;
  diagnosticText: string;
  ignorePorts: number[];
  protectPorts: number[];
  projectPathPrefixes: string[];
  onRefresh: () => void;
  onTerminate: (ports: PortUsage[]) => void;
  onNotify: (message: string, type: "info" | "success" | "error") => void;
  onIgnorePortsChange: (ports: number[]) => void;
  onProtectPortsChange: (ports: number[]) => void;
  onProjectPathPrefixesChange: (prefixes: string[]) => void;
}) {
  const [expandedPorts, setExpandedPorts] = useState<Set<number>>(new Set());
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [ignoreDraft, setIgnoreDraft] = useState(ignorePorts.join(", "));
  const [protectDraft, setProtectDraft] = useState(protectPorts.join(", "));
  const [prefixDraft, setPrefixDraft] = useState(projectPathPrefixes.join("\n"));
  const [listMode, setListMode] = useState<"port" | "project">("port");
  const metrics = portMetrics(ports);
  const groups = groupPorts(visiblePorts);
  const projectGroups = groupPortsByWorkingDirectory(visiblePorts);
  const busy = loading || refreshing;

  // 偏好异步加载后同步 draft，避免 blur 把空值写回。
  useEffect(() => {
    setIgnoreDraft(ignorePorts.join(", "));
  }, [ignorePorts]);

  useEffect(() => {
    setProtectDraft(protectPorts.join(", "));
  }, [protectPorts]);

  useEffect(() => {
    setPrefixDraft(projectPathPrefixes.join("\n"));
  }, [projectPathPrefixes]);

  function toggleSelected(id: string) {
    setSelectedIds((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  function terminateSelected() {
    const selected = visiblePorts.filter((port) => selectedIds.has(port.id));
    if (selected.length === 0) {
      onNotify("请先勾选要关闭的端口", "info");
      return;
    }
    onTerminate(selected);
    setSelectedIds(new Set());
  }

  async function openInBrowser(usage: PortUsage) {
    const url = loopbackBrowserUrl(usage);
    if (!url) {
      onNotify("仅支持在浏览器打开 loopback 地址", "error");
      return;
    }
    try {
      await openUrl(url);
      onNotify(`已打开 ${url}`, "success");
    } catch (error) {
      onNotify(`打开浏览器失败：${String(error)}`, "error");
    }
  }

  function togglePort(port: number) {
    setExpandedPorts((current) => {
      const next = new Set(current);
      if (next.has(port)) next.delete(port);
      else next.add(port);
      return next;
    });
  }

  async function copyPath(path: string) {
    if (!path) return;
    try {
      await navigator.clipboard.writeText(path);
      onNotify("路径已复制", "success");
    } catch (error) {
      onNotify(`路径复制失败：${String(error)}`, "error");
    }
  }

  async function revealPath(path: string) {
    try {
      await revealItemInDir(path);
    } catch (error) {
      onNotify(`Finder 打开失败：${String(error)}`, "error");
    }
  }

  async function openProjectPath(path: string) {
    try {
      await openPath(path);
    } catch (error) {
      onNotify(`路径打开失败：${String(error)}`, "error");
    }
  }

  return (
    <section className="port-workspace">
      <div className="port-header">
        <div className="port-title-block">
          <h2>Port</h2>
          <p>本机开发服务端口、进程和监听状态</p>
        </div>
        <button
          type="button"
          className="icon-btn"
          data-spin={busy}
          disabled={busy}
          onClick={onRefresh}
          aria-label="刷新端口"
          title="刷新端口"
        >
          <Icon.Refresh />
        </button>
      </div>

      <div className="port-metrics" aria-label="端口统计">
        <Metric title={scope === "project" ? "项目端口" : "已列出"} value={visiblePorts.length} />
        <Metric title="TCP" value={metrics.tcp} />
        <Metric title="UDP" value={metrics.udp} />
        <Metric title="进程" value={scope === "project" ? metrics.projectProcessCount : metrics.processCount} />
      </div>

      <div className="port-diagnostic">
        <span>{lastUpdated}</span>
        <span>{diagnosticText}</span>
        {selectedIds.size > 0 && (
          <button
            type="button"
            className="port-close-btn"
            disabled={busy}
            onClick={terminateSelected}
          >
            关闭选中（{selectedIds.size}）
          </button>
        )}
      </div>

      <div className="port-rules" aria-label="端口规则">
        <label>
          忽略端口（逗号分隔，不展示）
          <input
            value={ignoreDraft}
            placeholder="例如 5000, 7000"
            onChange={(e) => setIgnoreDraft(e.target.value)}
            onBlur={() => {
              const ports = ignoreDraft
                .split(/[,，\s]+/)
                .map((part) => Number(part.trim()))
                .filter((n) => Number.isFinite(n) && n > 0 && n <= 65535);
              setIgnoreDraft(ports.join(", "));
              onIgnorePortsChange(ports);
            }}
          />
        </label>
        <label>
          保护端口（逗号分隔，终止时整批拦截）
          <input
            value={protectDraft}
            placeholder="例如 3000, 5432"
            onChange={(e) => setProtectDraft(e.target.value)}
            onBlur={() => {
              const ports = protectDraft
                .split(/[,，\s]+/)
                .map((part) => Number(part.trim()))
                .filter((n) => Number.isFinite(n) && n > 0 && n <= 65535);
              setProtectDraft(ports.join(", "));
              onProtectPortsChange(ports);
            }}
          />
        </label>
        <label>
          项目路径前缀（每行一个，扩大「项目服务」）
          <textarea
            rows={2}
            value={prefixDraft}
            placeholder="/Users/me/codes"
            onChange={(e) => setPrefixDraft(e.target.value)}
            onBlur={() => {
              const prefixes = prefixDraft
                .split("\n")
                .map((line) => line.trim())
                .filter(Boolean);
              setPrefixDraft(prefixes.join("\n"));
              onProjectPathPrefixesChange(prefixes);
            }}
          />
        </label>
      </div>

      <div className="port-list-mode" aria-label="列表分组">
        <button
          type="button"
          className="btn"
          data-active={listMode === "port"}
          onClick={() => setListMode("port")}
        >
          按端口
        </button>
        <button
          type="button"
          className="btn"
          data-active={listMode === "project"}
          onClick={() => setListMode("project")}
        >
          按项目目录
        </button>
      </div>

      {listMode === "project" && projectGroups.length > 0 && (
        <div className="port-project-groups" aria-label="按项目分组">
          {projectGroups.map((group) => {
            const closable = group.usages.filter((u) => u.userOwned);
            return (
              <div className="port-project-group" key={group.workingDirectory || "__unknown__"}>
                <div className="port-project-group-head">
                  <div>
                    <strong title={group.workingDirectory || undefined}>
                      {group.label}
                    </strong>
                    <span className="muted"> · {group.usages.length} 端口</span>
                  </div>
                  {!group.isUnknown && closable.length > 0 && (
                    <button
                      type="button"
                      className="port-close-btn"
                      disabled={busy}
                      onClick={() => onTerminate(closable)}
                    >
                      关闭此项目端口（{closable.length}）
                    </button>
                  )}
                  {group.isUnknown && (
                    <span className="muted">未知目录无一键关闭</span>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}

      {groups.length === 0 && !busy ? (
        <p className="state-line">
          {scope === "project" ? "没有找到项目服务端口" : "没有找到端口占用"}
        </p>
      ) : (
        <div className="port-table" aria-label="端口列表">
          <div className="port-row port-row-head">
            <span />
            <span />
            <span>端口</span>
            <span>协议</span>
            <span>进程</span>
            <span>项目</span>
            <span>PID</span>
            <span>地址</span>
            <span>状态</span>
            <span />
          </div>
          {groups.map((group) => {
            const summary = groupSummary(group);
            const expanded = expandedPorts.has(group.port);
            const multiple = group.usages.length > 1;
            const primary = group.usages[0];
            return (
              <div className="port-group" key={group.port}>
                <PortRow
                  expanded={expanded}
                  multiple={multiple}
                  port={group.port}
                  protocol={summary.protocol}
                  process={summary.process}
                  executablePath={summary.executablePath}
                  project={summary.project}
                  projectPath={summary.project.startsWith("/") ? summary.project : ""}
                  pid={summary.pid}
                  address={summary.address}
                  state={summary.state}
                  busy={group.usages.some((usage) => terminatingIds.has(usage.id))}
                  selected={group.usages.every((usage) => selectedIds.has(usage.id))}
                  browserUrl={primary ? loopbackBrowserUrl(primary) : null}
                  onToggle={() => togglePort(group.port)}
                  onToggleSelect={() => {
                    const allSelected = group.usages.every((usage) =>
                      selectedIds.has(usage.id),
                    );
                    setSelectedIds((current) => {
                      const next = new Set(current);
                      for (const usage of group.usages) {
                        if (allSelected) next.delete(usage.id);
                        else next.add(usage.id);
                      }
                      return next;
                    });
                  }}
                  onTerminate={() => onTerminate(group.usages)}
                  onOpenBrowser={
                    primary ? () => void openInBrowser(primary) : undefined
                  }
                  onCopyPath={copyPath}
                  onRevealPath={revealPath}
                  onOpenPath={openProjectPath}
                />
                {multiple && expanded &&
                  group.usages.map((usage) => (
                    <PortDetailRow
                      key={usage.id}
                      usage={usage}
                      busy={terminatingIds.has(usage.id)}
                      selected={selectedIds.has(usage.id)}
                      onToggleSelect={() => toggleSelected(usage.id)}
                      onTerminate={() => onTerminate([usage])}
                      onOpenBrowser={() => void openInBrowser(usage)}
                      onCopyPath={copyPath}
                      onRevealPath={revealPath}
                      onOpenPath={openProjectPath}
                    />
                  ))}
              </div>
            );
          })}
        </div>
      )}
    </section>
  );
}

function Metric({ title, value }: { title: string; value: number }) {
  return (
    <span className="port-metric">
      <span>{title}</span>
      <strong>{value}</strong>
    </span>
  );
}

function PortRow({
  expanded,
  multiple,
  port,
  protocol,
  process,
  executablePath,
  project,
  projectPath,
  pid,
  address,
  state,
  busy,
  selected,
  browserUrl,
  onToggle,
  onToggleSelect,
  onTerminate,
  onOpenBrowser,
  onCopyPath,
  onRevealPath,
  onOpenPath,
}: {
  expanded: boolean;
  multiple: boolean;
  port: number;
  protocol: string;
  process: string;
  executablePath: string;
  project: string;
  projectPath: string;
  pid: string;
  address: string;
  state: string;
  busy: boolean;
  selected: boolean;
  browserUrl: string | null;
  onToggle: () => void;
  onToggleSelect: () => void;
  onTerminate: () => void;
  onOpenBrowser?: () => void;
  onCopyPath: (path: string) => void;
  onRevealPath: (path: string) => void;
  onOpenPath: (path: string) => void;
}) {
  return (
    <div className="port-row">
      <button type="button" className="port-disclosure" disabled={!multiple} data-open={expanded} onClick={onToggle}>
        {multiple ? <Icon.Chevron /> : null}
      </button>
      <label className="port-select">
        <input type="checkbox" checked={selected} onChange={onToggleSelect} />
      </label>
      <span className="port-number">{port}</span>
      <span className="port-protocol">{protocol}</span>
      <span className="port-process" title={executablePath || process}>{process}</span>
      <PathActions
        path={projectPath}
        label={projectPath ? shortPath(projectPath) : project}
        onCopyPath={onCopyPath}
        onRevealPath={onRevealPath}
        onOpenPath={onOpenPath}
      />
      <span className="port-pid">{pid}</span>
      <span className="port-address">{address}</span>
      <span className="port-state">{state || "-"}</span>
      <div className="port-row-actions">
        <button
          type="button"
          className="port-close-btn"
          disabled={!browserUrl || busy}
          title={browserUrl ? `打开 ${browserUrl}` : "非 loopback，无法打开浏览器"}
          onClick={onOpenBrowser}
        >
          打开
        </button>
        <button type="button" className="port-close-btn" disabled={busy} onClick={onTerminate}>
          {busy ? "关闭中" : multiple ? "关闭全部" : "关闭"}
        </button>
      </div>
    </div>
  );
}

function PortDetailRow({
  usage,
  busy,
  selected,
  onToggleSelect,
  onTerminate,
  onOpenBrowser,
  onCopyPath,
  onRevealPath,
  onOpenPath,
}: {
  usage: PortUsage;
  busy: boolean;
  selected: boolean;
  onToggleSelect: () => void;
  onTerminate: () => void;
  onOpenBrowser: () => void;
  onCopyPath: (path: string) => void;
  onRevealPath: (path: string) => void;
  onOpenPath: (path: string) => void;
}) {
  const browserUrl = loopbackBrowserUrl(usage);
  return (
    <div className="port-row port-row-detail">
      <span />
      <label className="port-select">
        <input type="checkbox" checked={selected} onChange={onToggleSelect} />
      </label>
      <span className="port-protocol">{protocolLabel(usage.protocol)}</span>
      <span className="port-process" title={usage.executablePath || usage.command}>{portProcessLabel(usage)}</span>
      <PathActions
        path={usage.workingDirectory}
        label={shortPath(usage.workingDirectory)}
        onCopyPath={onCopyPath}
        onRevealPath={onRevealPath}
        onOpenPath={onOpenPath}
      />
      <span className="port-pid">{usage.pid}</span>
      <span className="port-address" title={serverURLLabel(usage)}>{usage.address}</span>
      <span className="port-state">{usage.state || "-"}</span>
      <div className="port-row-actions">
        <button
          type="button"
          className="port-close-btn"
          disabled={!browserUrl || busy}
          title={browserUrl ? `打开 ${browserUrl}` : "非 loopback，无法打开浏览器"}
          onClick={onOpenBrowser}
        >
          打开
        </button>
        <button type="button" className="port-close-btn" disabled={busy} onClick={onTerminate}>
          {busy ? "关闭中" : "关闭"}
        </button>
      </div>
    </div>
  );
}

function PathActions({
  path,
  label,
  onCopyPath,
  onRevealPath,
  onOpenPath,
}: {
  path: string;
  label: string;
  onCopyPath: (path: string) => void;
  onRevealPath: (path: string) => void;
  onOpenPath: (path: string) => void;
}) {
  if (!path) {
    return <span className="port-path empty">-</span>;
  }

  return (
    <span className="port-path-actions">
      <button type="button" className="port-path" title={path} onClick={() => void onRevealPath(path)}>
        {label}
      </button>
      <button type="button" className="port-path-action" title="拷贝路径" onClick={() => void onCopyPath(path)}>
        复制
      </button>
      <button type="button" className="port-path-action" title="打开路径" onClick={() => void onOpenPath(path)}>
        打开
      </button>
    </span>
  );
}
