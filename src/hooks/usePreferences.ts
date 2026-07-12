import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  LAUNCH_MODE_LABELS,
  LaunchMode,
  StatusType,
  TERMINAL_LABELS,
  TerminalType,
  THEME_MODE_LABELS,
  ThemeMode,
} from "../types";

type NotifyStatus = (message: string, type: StatusType) => void;

export function usePreferences(notifyStatus: NotifyStatus) {
  const [availableTerminals, setAvailableTerminals] = useState<TerminalType[]>([
    "system",
  ]);
  const [preferredTerminal, setPreferredTerminal] =
    useState<TerminalType>("system");
  const [launchMode, setLaunchMode] = useState<LaunchMode>("new-tab");
  const [themeMode, setThemeMode] = useState<ThemeMode>("system");
  const [favoriteProjectDirs, setFavoriteProjectDirs] = useState<string[]>([]);
  const [portAutoRefresh, setPortAutoRefresh] = useState(true);

  async function loadPreferences() {
    const [available, preferred, mode, theme, favorites, autoRefresh] = await Promise.all([
      invoke<TerminalType[]>("list_available_terminals"),
      invoke<TerminalType>("get_preferred_terminal"),
      invoke<LaunchMode>("get_launch_mode"),
      invoke<ThemeMode>("get_theme_mode"),
      invoke<string[]>("get_favorite_project_dirs"),
      invoke<boolean>("get_port_auto_refresh"),
    ]);
    setAvailableTerminals(available);
    const resolved = available.includes(preferred)
      ? preferred
      : (available[0] ?? "system");
    setPreferredTerminal(resolved);
    if (resolved !== preferred) {
      await invoke("set_preferred_terminal", { terminal: resolved });
    }
    setLaunchMode(mode);
    setThemeMode(theme);
    setFavoriteProjectDirs(favorites);
    setPortAutoRefresh(autoRefresh);
  }

  async function handleTerminalChange(terminal: TerminalType) {
    await invoke("set_preferred_terminal", { terminal });
    setPreferredTerminal(terminal);
    notifyStatus(`终端已切换为 ${TERMINAL_LABELS[terminal]}`, "info");
  }

  async function handleLaunchModeChange(mode: LaunchMode) {
    await invoke("set_launch_mode", { mode });
    setLaunchMode(mode);
    notifyStatus(`打开方式已切换为${LAUNCH_MODE_LABELS[mode]}`, "info");
  }

  async function handleThemeModeChange(mode: ThemeMode) {
    await invoke("set_theme_mode", { mode });
    setThemeMode(mode);
    notifyStatus(`主题已切换为${THEME_MODE_LABELS[mode]}`, "info");
  }

  async function handleFavoriteProjectDirsChange(projectDirs: string[]) {
    const previous = favoriteProjectDirs;
    setFavoriteProjectDirs(projectDirs);
    try {
      await invoke("set_favorite_project_dirs", { projectDirs });
    } catch (error) {
      setFavoriteProjectDirs(previous);
      notifyStatus(`收藏保存失败：${String(error)}`, "error");
    }
  }

  async function handlePortAutoRefreshChange(enabled: boolean) {
    const previous = portAutoRefresh;
    setPortAutoRefresh(enabled);
    try {
      await invoke("set_port_auto_refresh", { enabled });
      notifyStatus(`端口自动刷新已${enabled ? "开启" : "关闭"}`, "info");
    } catch (error) {
      setPortAutoRefresh(previous);
      notifyStatus(`端口自动刷新保存失败：${String(error)}`, "error");
    }
  }

  return {
    availableTerminals,
    preferredTerminal,
    launchMode,
    themeMode,
    favoriteProjectDirs,
    portAutoRefresh,
    loadPreferences,
    handleTerminalChange,
    handleLaunchModeChange,
    handleThemeModeChange,
    handleFavoriteProjectDirsChange,
    handlePortAutoRefreshChange,
  };
}
