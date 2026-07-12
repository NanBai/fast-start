import { useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { PortScanResponse, PortUsage, StatusType } from "../types";

type NotifyStatus = (message: string, type: StatusType) => void;

export function usePorts(notifyStatus: NotifyStatus) {
  const [ports, setPorts] = useState<PortUsage[]>([]);
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [terminatingIds, setTerminatingIds] = useState<Set<string>>(new Set());
  const [pendingTerminate, setPendingTerminate] = useState<PortUsage[] | null>(null);
  const [lastScan, setLastScan] = useState<PortScanResponse | null>(null);
  const scanInFlight = useRef(false);
  const lastSilentError = useRef<string | null>(null);

  function applyPortResult(result: PortScanResponse, showStatus: boolean) {
    setPorts(result.ports);
    setLastScan(result);
    lastSilentError.current = null;
    if (showStatus) {
      notifyStatus(`已加载 ${result.ports.length} 个端口`, "success");
    }
  }

  async function loadPorts() {
    if (scanInFlight.current) return;
    scanInFlight.current = true;
    setLoading(true);
    try {
      const result = await invoke<PortScanResponse>("scan_ports");
      applyPortResult(result, true);
    } catch (error) {
      notifyStatus(`端口扫描失败：${String(error)}`, "error");
    } finally {
      setLoading(false);
      scanInFlight.current = false;
    }
  }

  async function refreshPorts(showStatus = true) {
    if (scanInFlight.current) return;
    scanInFlight.current = true;
    if (showStatus) setRefreshing(true);
    try {
      const result = await invoke<PortScanResponse>("refresh_ports");
      applyPortResult(result, showStatus);
    } catch (error) {
      const message = `端口扫描失败：${String(error)}`;
      if (showStatus) {
        notifyStatus(message, "error");
      } else if (lastSilentError.current !== message) {
        lastSilentError.current = message;
        notifyStatus(message, "error");
      }
    } finally {
      if (showStatus) setRefreshing(false);
      scanInFlight.current = false;
    }
  }

  function requestTerminatePorts(usages: PortUsage[]) {
    setPendingTerminate(usages);
  }

  function cancelTerminatePorts() {
    setPendingTerminate(null);
  }

  async function confirmTerminatePorts() {
    if (!pendingTerminate) return;
    const portIds = pendingTerminate.map((port) => port.id);
    setTerminatingIds(new Set(portIds));
    notifyStatus("正在关闭端口服务…", "info");
    try {
      const result = await invoke<PortScanResponse>("terminate_port_processes", { portIds });
      applyPortResult(result, true);
      notifyStatus("端口服务已关闭", "success");
      setPendingTerminate(null);
    } catch (error) {
      notifyStatus(`关闭失败：${String(error)}`, "error");
    } finally {
      setTerminatingIds(new Set());
    }
  }

  return {
    ports,
    loading,
    refreshing,
    terminatingIds,
    pendingTerminate,
    lastScan,
    loadPorts,
    refreshPorts,
    requestTerminatePorts,
    cancelTerminatePorts,
    confirmTerminatePorts,
  };
}
