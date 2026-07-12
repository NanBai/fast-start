import { useState } from "react";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
import { Icon } from "./icons/Icon";
import {
  groupPorts,
  groupSummary,
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
  onRefresh,
  onTerminate,
  onNotify,
}: {
  ports: PortUsage[];
  visiblePorts: PortUsage[];
  scope: PortScope;
  loading: boolean;
  refreshing: boolean;
  terminatingIds: Set<string>;
  lastUpdated: string;
  diagnosticText: string;
  onRefresh: () => void;
  onTerminate: (ports: PortUsage[]) => void;
  onNotify: (message: string, type: "info" | "success" | "error") => void;
}) {
  const [expandedPorts, setExpandedPorts] = useState<Set<number>>(new Set());
  const metrics = portMetrics(ports);
  const groups = groupPorts(visiblePorts);
  const busy = loading || refreshing;

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
      </div>

      {groups.length === 0 && !busy ? (
        <p className="state-line">
          {scope === "project" ? "没有找到项目服务端口" : "没有找到端口占用"}
        </p>
      ) : (
        <div className="port-table" aria-label="端口列表">
          <div className="port-row port-row-head">
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
                  onToggle={() => togglePort(group.port)}
                  onTerminate={() => onTerminate(group.usages)}
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
                      onTerminate={() => onTerminate([usage])}
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
  onToggle,
  onTerminate,
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
  onToggle: () => void;
  onTerminate: () => void;
  onCopyPath: (path: string) => void;
  onRevealPath: (path: string) => void;
  onOpenPath: (path: string) => void;
}) {
  return (
    <div className="port-row">
      <button type="button" className="port-disclosure" disabled={!multiple} data-open={expanded} onClick={onToggle}>
        {multiple ? <Icon.Chevron /> : null}
      </button>
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
      <button type="button" className="port-close-btn" disabled={busy} onClick={onTerminate}>
        {busy ? "关闭中" : multiple ? "关闭全部" : "关闭"}
      </button>
    </div>
  );
}

function PortDetailRow({
  usage,
  busy,
  onTerminate,
  onCopyPath,
  onRevealPath,
  onOpenPath,
}: {
  usage: PortUsage;
  busy: boolean;
  onTerminate: () => void;
  onCopyPath: (path: string) => void;
  onRevealPath: (path: string) => void;
  onOpenPath: (path: string) => void;
}) {
  return (
    <div className="port-row port-row-detail">
      <span />
      <span />
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
      <button type="button" className="port-close-btn" disabled={busy} onClick={onTerminate}>
        {busy ? "关闭中" : "关闭"}
      </button>
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
