import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  OmpConfigHealth,
  OmpProvider,
  OmpSetRoleResult,
  StatusType,
} from "../types";

type NotifyStatus = (message: string, type: StatusType) => void;

export function useOmpProviders(notifyStatus: NotifyStatus) {
  const [providers, setProviders] = useState<OmpProvider[]>([]);
  const [health, setHealth] = useState<OmpConfigHealth | null>(null);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState(false);

  async function refresh(showStatus = true) {
    setLoading(true);
    try {
      const [provResult, healthResult] = await Promise.all([
        invoke<{ providers: OmpProvider[]; note?: string }>("omp_list_providers").catch(() => ({
          providers: [],
          note: "无法获取",
        })),
        invoke<OmpConfigHealth>("omp_get_config_health").catch(() => null),
      ]);

      setProviders(provResult.providers || []);
      setHealth(healthResult);

      if (showStatus) {
        notifyStatus("Oh My Pi 供应商信息已刷新", "info");
      }
    } catch (e) {
      notifyStatus(`刷新 Oh My Pi 失败: ${e}`, "error");
    } finally {
      setLoading(false);
    }
  }

  async function setRoleModel(role: string, model: string): Promise<boolean> {
    setBusy(true);
    try {
      const result = await invoke<OmpSetRoleResult>("omp_set_role_model", {
        role,
        model,
      });

      if (result.ok) {
        notifyStatus(`已设置 ${role} 为 ${model}${result.backup ? "（已备份）" : ""}`, "success");
        // 刷新健康状态
        await refresh(false);
        return true;
      } else {
        notifyStatus(result.message || "设置失败", "error");
        return false;
      }
    } catch (e) {
      notifyStatus(`设置模型失败: ${e}`, "error");
      return false;
    } finally {
      setBusy(false);
    }
  }

  return {
    providers,
    health,
    loading,
    busy,
    refresh,
    setRoleModel,
  };
}
