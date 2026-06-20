import { useState } from "react";
import { Icon } from "./icons/Icon";
import { recentDaysLabel, RECENT_DAY_OPTIONS, RecentDaysFilter } from "../lib/sessionUtils";
import {
  LAUNCH_MODE_LABELS,
  LaunchMode,
  TERMINAL_LABELS,
  TerminalType,
  THEME_MODE_LABELS,
  THEME_MODE_OPTIONS,
  ThemeMode,
} from "../types";

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
        <span className="menu-dot range-dot" />
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
    <label className="menu">
      <span className="menu-value">
        <span className="menu-dot" />
        {TERMINAL_LABELS[value]}
        <Icon.Chevron />
      </span>
      <select
        value={value}
        disabled={saving}
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
