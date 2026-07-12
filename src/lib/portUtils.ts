import { PORT_PROTOCOL_LABELS, PortProtocol, PortScope, PortUsage } from "../types";

export type PortFilterOptions = {
  scope: PortScope;
  protocol: PortProtocol | "all";
  query: string;
};

export type PortUsageGroup = {
  port: number;
  usages: PortUsage[];
};

export function filterPorts(ports: PortUsage[], options: PortFilterOptions) {
  const query = options.query.trim().toLowerCase();
  return ports.filter((port) => {
    if (options.scope === "project" && !port.isProjectService) return false;
    if (options.protocol !== "all" && port.protocol !== options.protocol) return false;
    if (!query) return true;
    return portMatchesQuery(port, query);
  });
}

export function groupPorts(ports: PortUsage[]): PortUsageGroup[] {
  const groups = new Map<number, PortUsage[]>();
  for (const port of ports) {
    const current = groups.get(port.port) ?? [];
    current.push(port);
    groups.set(port.port, current);
  }
  return Array.from(groups.entries())
    .map(([port, usages]) => ({ port, usages }))
    .sort((left, right) => left.port - right.port);
}

export function portProcessLabel(port: PortUsage) {
  if (
    port.isProjectService &&
    port.parentCommand &&
    port.parentCommand !== port.command &&
    !isShell(port.parentCommand)
  ) {
    return port.parentCommand;
  }
  return port.command || "-";
}

export function shortPath(path: string) {
  if (!path) return "-";
  const parts = path.split("/").filter(Boolean);
  const name = parts[parts.length - 1];
  return name ? `.../${name}` : path;
}

export function serverURLLabel(port: PortUsage) {
  return `${normalizedServerHost(port.address)}:${port.port}`;
}

export function protocolLabel(protocol: PortProtocol) {
  return PORT_PROTOCOL_LABELS[protocol];
}

export function groupSummary(group: PortUsageGroup) {
  const processes = unique(group.usages.map(portProcessLabel));
  const pids = unique(group.usages.map((port) => String(port.pid)));
  const protocols = unique(group.usages.map((port) => protocolLabel(port.protocol)));
  const addresses = unique(group.usages.map((port) => port.address));
  const states = unique(group.usages.map((port) => port.state || "-"));
  const dirs = unique(
    group.usages.map((port) => port.workingDirectory).filter((path) => path.length > 0),
  );

  return {
    process: processes.length === 1 ? processes[0] : `${processes.length} 个进程`,
    pid: pids.length === 1 ? pids[0] : `${pids.length} 项`,
    protocol: protocols.join(" / "),
    address: addresses.length === 1 ? addresses[0] : `${addresses.length} 个地址`,
    state: states.join(" / "),
    project: dirs.length === 0 ? "-" : dirs.length === 1 ? dirs[0] : `${dirs.length} 个目录`,
    executablePath: group.usages[0]?.executablePath ?? "",
  };
}

export function portMetrics(ports: PortUsage[]) {
  const projectPorts = ports.filter((port) => port.isProjectService);
  return {
    tcp: ports.filter((port) => port.protocol === "tcp").length,
    udp: ports.filter((port) => port.protocol === "udp").length,
    processCount: new Set(ports.map((port) => port.pid)).size,
    projectProcessCount: new Set(projectPorts.map((port) => port.pid)).size,
    projectCount: projectPorts.length,
  };
}

function portMatchesQuery(port: PortUsage, query: string) {
  return [
    port.command,
    port.parentCommand,
    port.user,
    port.address,
    port.workingDirectory,
    port.executablePath,
    String(port.port),
    String(port.pid),
  ]
    .join(" ")
    .toLowerCase()
    .includes(query);
}

function normalizedServerHost(address: string) {
  if (address === "*" || address === "0.0.0.0" || address === "::" || address === "[::]") {
    return "localhost";
  }
  if (address === "[::1]" || address === "::1") {
    return "127.0.0.1";
  }
  return address;
}

function unique(values: string[]) {
  return values.reduce<string[]>((result, value) => {
    if (!result.includes(value)) result.push(value);
    return result;
  }, []);
}

function isShell(command: string) {
  return new Set(["bash", "zsh", "fish", "sh", "dash", "tcsh", "ksh", "csh"]).has(
    command.toLowerCase(),
  );
}
