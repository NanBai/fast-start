import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  CliScanError,
  LaunchCommandPreview,
  RecentLaunch,
  ScanResponse,
  SessionData,
  StatusType,
} from "../types";

type NotifyStatus = (message: string, type: StatusType) => void;

function formatScanStatus(result: ScanResponse): { message: string; type: StatusType } {
  const base =
    result.scanErrors.length > 0
      ? `已加载 ${result.sessions.length} 个 session · ${result.scanErrors.length} 个 CLI 扫描失败`
      : `已加载 ${result.sessions.length} 个 session`;

  const parts: string[] = [base];
  if (result.fromCache === true) {
    parts.push("缓存");
  }
  if (typeof result.scanDurationMs === "number") {
    parts.push(`${result.scanDurationMs}ms`);
  }

  return {
    message: parts.join(" · "),
    type: result.scanErrors.length > 0 ? "error" : "success",
  };
}

export function useSessions(notifyStatus: NotifyStatus) {
  const [sessions, setSessions] = useState<SessionData[]>([]);
  const [scanErrors, setScanErrors] = useState<CliScanError[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [launchingId, setLaunchingId] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<SessionData | null>(null);
  const [recentLaunches, setRecentLaunches] = useState<RecentLaunch[]>([]);
  const [commandPreview, setCommandPreview] = useState<LaunchCommandPreview | null>(
    null,
  );

  function applyScanResult(result: ScanResponse) {
    setSessions(result.sessions);
    setScanErrors(result.scanErrors);
    const status = formatScanStatus(result);
    notifyStatus(status.message, status.type);
  }

  async function loadSessions() {
    setLoading(true);
    try {
      const result = await invoke<ScanResponse>("scan_sessions");
      applyScanResult(result);
      void loadRecentLaunches();
      // 缓存秒开后立即全量 refresh，补齐 delete_target 并写回 snapshot
      if (result.fromCache === true) {
        setLoading(false);
        await refreshSessions();
        return;
      }
    } catch (error) {
      notifyStatus(String(error), "error");
    } finally {
      setLoading(false);
    }
  }

  async function refreshSessions() {
    setRefreshing(true);
    try {
      const result = await invoke<ScanResponse>("refresh_sessions");
      applyScanResult(result);
    } catch (error) {
      notifyStatus(String(error), "error");
    } finally {
      setRefreshing(false);
    }
  }

  async function loadRecentLaunches() {
    try {
      const launches = await invoke<RecentLaunch[]>("get_recent_launches");
      setRecentLaunches(launches);
    } catch {
      // 非关键路径：启动历史失败不阻断列表
    }
  }

  async function launchSession(sessionId: string) {
    setLaunchingId(sessionId);
    notifyStatus("正在启动终端…", "info");
    try {
      // sessionListId = Session.id（列表稳定 id），不是 CLI 原始 sessionId
      await invoke("launch_session", { sessionListId: sessionId });
      notifyStatus("终端启动成功", "success");
      await loadRecentLaunches();
    } catch (error) {
      notifyStatus(`启动失败：${String(error)}`, "error");
    } finally {
      setLaunchingId(null);
    }
  }

  async function previewLaunchCommand(sessionId: string) {
    try {
      const preview = await invoke<LaunchCommandPreview>("preview_launch_command", {
        sessionListId: sessionId,
      });
      setCommandPreview(preview);
      notifyStatus(
        `命令预览：${preview.program} ${preview.args.join(" ")}`,
        "info",
      );
      return preview;
    } catch (error) {
      notifyStatus(`预览失败：${String(error)}`, "error");
      return null;
    }
  }

  function clearCommandPreview() {
    setCommandPreview(null);
  }

  function requestDeleteSession(session: SessionData) {
    setPendingDelete(session);
  }

  function cancelDeleteSession() {
    setPendingDelete(null);
  }

  async function confirmDeleteSession() {
    if (!pendingDelete) {
      return;
    }
    setDeletingId(pendingDelete.id);
    notifyStatus("正在删除 session…", "info");
    try {
      const result = await invoke<ScanResponse>("delete_session", {
        sessionListId: pendingDelete.id,
      });
      applyScanResult(result);
      await loadRecentLaunches();
      notifyStatus("session 已删除", "success");
      setPendingDelete(null);
    } catch (error) {
      notifyStatus(`删除失败：${String(error)}`, "error");
    } finally {
      setDeletingId(null);
    }
  }

  return {
    sessions,
    scanErrors,
    loading,
    refreshing,
    launchingId,
    deletingId,
    pendingDelete,
    recentLaunches,
    commandPreview,
    loadSessions,
    refreshSessions,
    launchSession,
    previewLaunchCommand,
    clearCommandPreview,
    loadRecentLaunches,
    requestDeleteSession,
    cancelDeleteSession,
    confirmDeleteSession,
  };
}
