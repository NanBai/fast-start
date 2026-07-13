import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  emptyGrokProfile,
  GrokActivateOfficialResult,
  GrokBackupInfo,
  GrokPrivacyResult,
  GrokProfile,
  GrokProviderLayout,
  GrokProviderStatus,
  StatusType,
} from "../types";

type NotifyStatus = (message: string, type: StatusType) => void;

const emptyLayout = (): GrokProviderLayout => ({ order: [], pinnedIds: [] });

export function useGrokProviders(notifyStatus: NotifyStatus) {
  const [profiles, setProfiles] = useState<GrokProfile[]>([]);
  const [status, setStatus] = useState<GrokProviderStatus | null>(null);
  const [backups, setBackups] = useState<GrokBackupInfo[]>([]);
  const [layout, setLayout] = useState<GrokProviderLayout>(emptyLayout());
  const [loading, setLoading] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);

  async function refreshAll(showStatus = true) {
    setLoading(true);
    try {
      const [nextStatus, nextProfiles, nextBackups, layoutResult] = await Promise.all([
        invoke<GrokProviderStatus>("grok_provider_status"),
        invoke<GrokProfile[]>("grok_list_profiles"),
        invoke<GrokBackupInfo[]>("grok_list_backups"),
        invoke<GrokProviderLayout>("get_grok_provider_layout").then(
          (layout) => ({ ok: true as const, layout }),
          (error) => ({ ok: false as const, error }),
        ),
      ]);
      setStatus(nextStatus);
      setProfiles(nextProfiles);
      setBackups(nextBackups);
      if (layoutResult.ok) {
        setLayout(layoutResult.layout);
      } else {
        setLayout(emptyLayout());
        notifyStatus(`卡片顺序加载失败，已用默认顺序：${String(layoutResult.error)}`, "error");
      }
      if (showStatus) {
        notifyStatus(`已加载 ${nextProfiles.length} 个 Grok 供应商`, "success");
      }
    } catch (error) {
      notifyStatus(`Grok 供应商加载失败：${String(error)}`, "error");
    } finally {
      setLoading(false);
    }
  }

  async function activate(id: string) {
    setBusyId(id);
    try {
      await invoke<GrokProfile>("grok_activate_profile", { id });
      notifyStatus("已启用供应商（新开 Grok 会话生效）", "success");
      await refreshAll(false);
    } catch (error) {
      notifyStatus(`启用失败：${String(error)}`, "error");
    } finally {
      setBusyId(null);
    }
  }

  async function activateOfficial() {
    setBusyId("official");
    try {
      const result = await invoke<GrokActivateOfficialResult>("grok_activate_official");
      notifyStatus(result.message, "success");
      await refreshAll(false);
    } catch (error) {
      notifyStatus(`切换官方账号失败：${String(error)}`, "error");
    } finally {
      setBusyId(null);
    }
  }

  async function applyPrivacy() {
    setBusyId("privacy");
    try {
      const result = await invoke<GrokPrivacyResult>("grok_apply_privacy_protection");
      notifyStatus(result.message, "success");
      await refreshAll(false);
    } catch (error) {
      notifyStatus(`隐私保护写入失败：${String(error)}`, "error");
    } finally {
      setBusyId(null);
    }
  }

  async function saveLayout(next: GrokProviderLayout) {
    try {
      const saved = await invoke<GrokProviderLayout>("set_grok_provider_layout", {
        layout: next,
      });
      setLayout(saved);
      return saved;
    } catch (error) {
      notifyStatus(`保存卡片顺序失败：${String(error)}`, "error");
      return null;
    }
  }

  async function importCurrent() {
    setBusyId("import");
    try {
      await invoke<GrokProfile>("grok_import_current", {
        name: "Default",
        active: true,
      });
      notifyStatus("已从 config.toml 导入并启用", "success");
      await refreshAll(false);
    } catch (error) {
      notifyStatus(`导入失败：${String(error)}`, "error");
    } finally {
      setBusyId(null);
    }
  }

  async function saveProfile(profile: GrokProfile, activateAfter: boolean) {
    setBusyId(profile.id || "new");
    try {
      let saved: GrokProfile;
      if (profile.id) {
        saved = await invoke<GrokProfile>("grok_update_profile", {
          id: profile.id,
          profile,
        });
      } else {
        saved = await invoke<GrokProfile>("grok_create_profile", { profile });
      }
      if (activateAfter) {
        await invoke("grok_activate_profile", { id: saved.id });
        notifyStatus("已保存并启用供应商", "success");
      } else {
        notifyStatus("供应商已保存", "success");
      }
      await refreshAll(false);
      return saved;
    } catch (error) {
      notifyStatus(`保存失败：${String(error)}`, "error");
      return null;
    } finally {
      setBusyId(null);
    }
  }

  async function removeProfile(id: string) {
    setBusyId(id);
    try {
      await invoke("grok_delete_profile", { id });
      notifyStatus("供应商已删除", "success");
      await refreshAll(false);
    } catch (error) {
      notifyStatus(`删除失败：${String(error)}`, "error");
    } finally {
      setBusyId(null);
    }
  }

  async function restoreBackup(file: string) {
    setBusyId(file);
    try {
      await invoke("grok_restore_backup", { file });
      notifyStatus("已还原备份（新开 Grok 会话生效）", "success");
      await refreshAll(false);
    } catch (error) {
      notifyStatus(`还原失败：${String(error)}`, "error");
    } finally {
      setBusyId(null);
    }
  }

  return {
    profiles,
    status,
    backups,
    layout,
    loading,
    busyId,
    refreshAll,
    activate,
    activateOfficial,
    applyPrivacy,
    saveLayout,
    importCurrent,
    saveProfile,
    removeProfile,
    restoreBackup,
    emptyGrokProfile,
  };
}
