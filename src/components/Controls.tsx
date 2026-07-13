import { useState } from "react";
import type { KeyboardEvent, RefObject } from "react";
import { Icon } from "./icons/Icon";
import { recentDaysLabel, RECENT_DAY_OPTIONS, RecentDaysFilter } from "../lib/sessionUtils";
import {
  LAUNCH_MODE_LABELS,
  LaunchMode,
  PORT_PROTOCOL_LABELS,
  PORT_SCOPE_LABELS,
  PortProtocol,
  PortScope,
  SESSION_LIST_MODE_LABELS,
  SessionListMode,
  TERMINAL_LABELS,
  TerminalType,
  THEME_MODE_LABELS,
  THEME_MODE_OPTIONS,
  ThemeMode,
} from "../types";

export function SearchBox({
  value,
  onChange,
  inputRef,
  onKeyDown,
  placeholder = "搜索 session、项目或路径",
  ariaLabel = "搜索 session",
}: {
  value: string;
  onChange: (value: string) => void;
  inputRef?: RefObject<HTMLInputElement | null>;
  onKeyDown?: (event: KeyboardEvent<HTMLInputElement>) => void;
  placeholder?: string;
  ariaLabel?: string;
}) {
  return (
    <label className="search-box">
      <Icon.Search />
      <input
        ref={inputRef}
        type="search"
        value={value}
        aria-label={ariaLabel}
        placeholder={placeholder}
        onKeyDown={onKeyDown}
        onChange={(event) => onChange(event.target.value)}
      />
      {value && (
        <button
          type="button"
          className="search-clear"
          aria-label="清空搜索"
          onClick={() => onChange("")}
        >
          <Icon.Close />
        </button>
      )}
    </label>
  );
}

export function PortScopeSegmented({
  value,
  onChange,
}: {
  value: PortScope;
  onChange: (scope: PortScope) => void;
}) {
  return (
    <div className="segmented port-scope-segmented" data-mode={value}>
      {(["project", "all"] as PortScope[]).map((scope) => (
        <button
          key={scope}
          type="button"
          className="segment"
          data-active={value === scope}
          onClick={() => onChange(scope)}
        >
          {scope === "project" ? <Icon.Project /> : <Icon.Globe />}
          {PORT_SCOPE_LABELS[scope]}
        </button>
      ))}
    </div>
  );
}

export function SessionListModeSegmented({
  value,
  onChange,
}: {
  value: SessionListMode;
  onChange: (mode: SessionListMode) => void | Promise<void>;
}) {
  return (
    <div className="segmented session-list-mode-segmented" data-mode={value}>
      {(["by-agent", "by-project"] as SessionListMode[]).map((mode) => (
        <button
          key={mode}
          type="button"
          className="segment"
          data-active={value === mode}
          onClick={() => void onChange(mode)}
        >
          {mode === "by-agent" ? <Icon.Agent /> : <Icon.Project />}
          {SESSION_LIST_MODE_LABELS[mode]}
        </button>
      ))}
    </div>
  );
}

export function ProtocolMenu({
  value,
  onChange,
}: {
  value: PortProtocol | "all";
  onChange: (protocol: PortProtocol | "all") => void;
}) {
  return (
    <label className="menu protocol-menu">
      <span className="menu-value">
        <Icon.Network />
        {value === "all" ? "全部协议" : PORT_PROTOCOL_LABELS[value]}
        <Icon.Chevron />
      </span>
      <select
        value={value}
        aria-label="筛选端口协议"
        onChange={(event) => onChange(event.target.value as PortProtocol | "all")}
      >
        <option value="all">全部协议</option>
        <option value="tcp">TCP</option>
        <option value="udp">UDP</option>
      </select>
    </label>
  );
}

export function AutoRefreshToggle({
  enabled,
  onChange,
}: {
  enabled: boolean;
  onChange: (enabled: boolean) => Promise<void>;
}) {
  const [saving, setSaving] = useState(false);

  async function handleClick() {
    setSaving(true);
    try {
      await onChange(!enabled);
    } finally {
      setSaving(false);
    }
  }

  return (
    <button
      type="button"
      className="toggle-btn"
      data-active={enabled}
      disabled={saving}
      onClick={() => void handleClick()}
      aria-pressed={enabled}
    >
      <Icon.Refresh />
      <span className="toggle-track" />
      自动刷新
    </button>
  );
}

export function RecentDaysMenu({
  value,
  onChange,
  visibleCount,
  totalCount,
}: {
  value: RecentDaysFilter;
  onChange: (value: RecentDaysFilter) => void;
  visibleCount: number;
  totalCount: number;
}) {
  return (
    <label className="menu range-menu">
      <span className="menu-value">
        <Icon.Clock />
        {recentDaysLabel(value)}
        <span className="menu-count">{visibleCount}/{totalCount}</span>
        <Icon.Chevron />
      </span>
      <select
        value={value}
        aria-label="显示最近几天的 session"
        onChange={(event) => onChange(event.target.value as RecentDaysFilter)}
      >
        {RECENT_DAY_OPTIONS.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}

export function TerminalMenu({
  value,
  available,
  onChange,
}: {
  value: TerminalType;
  available: TerminalType[];
  onChange: (terminal: TerminalType) => Promise<void>;
}) {
  const [saving, setSaving] = useState(false);

  async function handleChange(next: TerminalType) {
    if (next === value || !available.includes(next)) return;
    setSaving(true);
    try {
      await onChange(next);
    } finally {
      setSaving(false);
    }
  }

  return (
    <label className="menu terminal-menu">
      <span className="menu-value">
        <Icon.Terminal />
        {TERMINAL_LABELS[value]}
        <Icon.Chevron />
      </span>
      <select
        value={value}
        disabled={saving}
        aria-label="选择终端"
        onChange={(event) => void handleChange(event.target.value as TerminalType)}
      >
        {(Object.keys(TERMINAL_LABELS) as TerminalType[]).map((terminal) => {
          const enabled = available.includes(terminal);
          return (
            <option key={terminal} value={terminal} disabled={!enabled}>
              {TERMINAL_LABELS[terminal]}
              {!enabled ? "（未安装）" : ""}
            </option>
          );
        })}
      </select>
    </label>
  );
}

export function ThemeMenu({
  value,
  onChange,
}: {
  value: ThemeMode;
  onChange: (mode: ThemeMode) => Promise<void>;
}) {
  const [saving, setSaving] = useState(false);

  async function handleChange(next: ThemeMode) {
    if (next === value) return;
    setSaving(true);
    try {
      await onChange(next);
    } finally {
      setSaving(false);
    }
  }

  return (
    <label className="menu theme-menu">
      <span className="menu-value">
        <Icon.Theme />
        <span className="menu-dot theme-dot" data-theme-mode={value} />
        {THEME_MODE_LABELS[value]}
        <Icon.Chevron />
      </span>
      <select
        value={value}
        disabled={saving}
        aria-label="选择主题"
        onChange={(event) => void handleChange(event.target.value as ThemeMode)}
      >
        {THEME_MODE_OPTIONS.map((mode) => (
          <option key={mode} value={mode}>
            {THEME_MODE_LABELS[mode]}
          </option>
        ))}
      </select>
    </label>
  );
}

export function LaunchSegmented({
  value,
  onChange,
}: {
  value: LaunchMode;
  onChange: (mode: LaunchMode) => Promise<void>;
}) {
  const [saving, setSaving] = useState(false);

  async function handleChange(next: LaunchMode) {
    if (next === value) return;
    setSaving(true);
    try {
      await onChange(next);
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="segmented" data-mode={value} aria-disabled={saving}>
      {(["new-tab", "new-window"] as LaunchMode[]).map((mode) => (
        <button
          key={mode}
          type="button"
          className="segment"
          data-active={value === mode}
          disabled={saving}
          onClick={() => void handleChange(mode)}
        >
          {mode === "new-tab" ? <Icon.Tab /> : <Icon.Window />}
          {LAUNCH_MODE_LABELS[mode]}
        </button>
      ))}
    </div>
  );
}
