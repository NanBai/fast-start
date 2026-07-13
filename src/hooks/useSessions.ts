import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  BulkDeleteResult,
  CliScanError,
  LaunchCommandPreview,
  PreflightResult,
  RecentLaunch,
  ScanResponse,
  SessionData,
  SessionHealth,
  SessionHealthReport,
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
  const [healthById, setHealthById] = useState<Map<string, SessionHealth>>(
    () => new Map(),
  );
  const [selectedIds, setSelectedIds] = useState<Set<string>>(() => new Set());
  const [pendingBulkDelete, setPendingBulkDelete] = useState(false);
  const [bulkDeleting, setBulkDeleting] = useState(false);

  function applyScanResult(result: ScanResponse) {
    setSessions(result.sessions);
    setScanErrors(result.scanErrors);
    // 列表变化后丢掉已不存在的选中 id
    setSelectedIds((prev) => {
      const allowed = new Set(result.sessions.map((s) => s.id));
      const next = new Set<string>();
      for (const id of prev) {
        if (allowed.has(id)) next.add(id);
      }
      return next;
    });
    const status = formatScanStatus(result);
    notifyStatus(status.message, status.type);
    // 扫描后异步探测可见量（≤200）；失败不阻断列表
    void inspectHealthForSessions(result.sessions);
  }

  function toggleSessionSelected(sessionId: string) {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(sessionId)) next.delete(sessionId);
      else next.add(sessionId);
      return next;
    });
  }

  function clearSessionSelection() {
    setSelectedIds(new Set());
  }

  function requestBulkDelete() {
    if (selectedIds.size === 0) {
      notifyStatus("请先勾选要删除的 session", "error");
      return;
    }
    if (selectedIds.size > 50) {
      notifyStatus("单次最多删除 50 条", "error");
      return;
    }
    setPendingBulkDelete(true);
  }

  function cancelBulkDelete() {
    setPendingBulkDelete(false);
  }

  async function confirmBulkDelete() {
    if (selectedIds.size === 0) {
      setPendingBulkDelete(false);
      return;
    }
    setBulkDeleting(true);
    notifyStatus(`正在批量删除 ${selectedIds.size} 条…`, "info");
    try {
      const result = await invoke<BulkDeleteResult>("delete_sessions", {
        sessionListIds: Array.from(selectedIds),
      });
      applyScanResult({
        sessions: result.sessions,
        scanErrors: result.scanErrors,
        fromCache: result.fromCache ?? undefined,
        scanDurationMs: result.scanDurationMs ?? undefined,
      });
      await loadRecentLaunches();
      clearSessionSelection();
      setPendingBulkDelete(false);
      if (result.failures.length === 0) {
        notifyStatus(`全部成功：已删除 ${result.deletedIds.length} 条`, "success");
      } else {
        const failText = result.failures
          .map((f) => f.message)
          .slice(0, 3)
          .join("；");
        notifyStatus(
          `已删 ${result.deletedIds.length} 条，失败 ${result.failures.length} 条：${failText}`,
          "error",
        );
      }
    } catch (error) {
      notifyStatus(`批量删除失败：${String(error)}`, "error");
    } finally {
      setBulkDeleting(false);
    }
  }

  async function inspectHealthForSessions(list: SessionData[]) {
    if (list.length === 0) {
      setHealthById(new Map());
      return;
    }
    const ids = list.slice(0, 200).map((s) => s.id);
    try {
      const report = await invoke<SessionHealthReport>("inspect_session_health", {
        sessionListIds: ids,
      });
      const next = new Map<string, SessionHealth>();
      for (const item of report.items) {
        next.set(item.sessionListId, item);
      }
      setHealthById(next);
    } catch {
      // 非关键路径
    }
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

  async function preflightLaunch(sessionId: string): Promise<PreflightResult | null> {
    try {
      return await invoke<PreflightResult>("preflight_launch", {
        sessionListId: sessionId,
      });
    } catch (error) {
      notifyStatus(`预检失败：${String(error)}`, "error");
      return null;
    }
  }

  async function launchSession(sessionId: string) {
    setLaunchingId(sessionId);
    notifyStatus("正在启动终端…", "info");
    try {
      // sessionListId = Session.id（列表稳定 id），不是 CLI 原始 sessionId
      // 先只读预检，把 block/warn 展示给用户；launch_session 仍会再跑同一门闩。
      const preflight = await preflightLaunch(sessionId);
      if (preflight && !preflight.ok) {
        const blocks = preflight.checks
          .filter((c) => c.severity === "block")
          .map((c) => c.message);
        notifyStatus(
          `启动失败：${blocks.length > 0 ? blocks.join("；") : "预检未通过"}`,
          "error",
        );
        return;
      }
      if (preflight) {
        const warns = preflight.checks
          .filter((c) => c.severity === "warn")
          .map((c) => c.message);
        if (warns.length > 0) {
          notifyStatus(`预检提示：${warns.join("；")}`, "info");
        }
      }
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
      // 优先走 preflight：同组装路径 + 可展示 checks
      const preflight = await preflightLaunch(sessionId);
      if (preflight?.preview) {
        setCommandPreview(preflight.preview);
        const warnText = preflight.checks
          .filter((c) => c.severity === "warn")
          .map((c) => c.message)
          .join("；");
        const blockText = preflight.checks
          .filter((c) => c.severity === "block")
          .map((c) => c.message)
          .join("；");
        if (blockText) {
          notifyStatus(`预览可用但预检未通过：${blockText}`, "error");
        } else if (warnText) {
          notifyStatus(
            `命令预览：${preflight.preview.program} ${preflight.preview.args.join(" ")} · ${warnText}`,
            "info",
          );
        } else {
          notifyStatus(
            `命令预览：${preflight.preview.program} ${preflight.preview.args.join(" ")}`,
            "info",
          );
        }
        return preflight.preview;
      }
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
    healthById,
    selectedIds,
    pendingBulkDelete,
    bulkDeleting,
    loadSessions,
    refreshSessions,
    launchSession,
    preflightLaunch,
    previewLaunchCommand,
    clearCommandPreview,
    loadRecentLaunches,
    requestDeleteSession,
    cancelDeleteSession,
    confirmDeleteSession,
    inspectHealthForSessions,
    toggleSessionSelected,
    clearSessionSelection,
    requestBulkDelete,
    cancelBulkDelete,
    confirmBulkDelete,
  };
}
