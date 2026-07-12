import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  emptyGrokProfile,
  GrokBackupInfo,
  GrokProfile,
  GrokProviderStatus,
  StatusType,
} from "../types";

type NotifyStatus = (message: string, type: StatusType) => void;

export function useGrokProviders(notifyStatus: NotifyStatus) {
  const [profiles, setProfiles] = useState<GrokProfile[]>([]);
  const [status, setStatus] = useState<GrokProviderStatus | null>(null);
  const [backups, setBackups] = useState<GrokBackupInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);

  async function refreshAll(showStatus = true) {
    setLoading(true);
    try {
      const [nextStatus, nextProfiles, nextBackups] = await Promise.all([
        invoke<GrokProviderStatus>("grok_provider_status"),
        invoke<GrokProfile[]>("grok_list_profiles"),
        invoke<GrokBackupInfo[]>("grok_list_backups"),
      ]);
      setStatus(nextStatus);
      setProfiles(nextProfiles);
      setBackups(nextBackups);
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
    loading,
    busyId,
    refreshAll,
    activate,
    importCurrent,
    saveProfile,
    removeProfile,
    restoreBackup,
    emptyGrokProfile,
  };
}
