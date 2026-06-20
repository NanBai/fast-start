import { SessionData } from "../types";

export type RecentDaysFilter = "1" | "3" | "7" | "14" | "30" | "all";

export const RECENT_DAY_OPTIONS: { value: RecentDaysFilter; label: string }[] = [
  { value: "1", label: "最近 1 天" },
  { value: "3", label: "最近 3 天" },
  { value: "7", label: "最近 7 天" },
  { value: "14", label: "最近 14 天" },
  { value: "30", label: "最近 30 天" },
  { value: "all", label: "全部" },
];

export function formatRelative(iso: string) {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  const diff = Date.now() - date.getTime();
  const min = Math.round(diff / 60000);
  const hour = Math.round(min / 60);
  const day = Math.round(hour / 24);
  if (min < 1) return "刚刚";
  if (min < 60) return `${min} 分钟前`;
  if (hour < 24) return `${hour} 小时前`;
  if (day < 30) return `${day} 天前`;
  return date.toLocaleDateString();
}

export function recentDaysLabel(value: RecentDaysFilter) {
  return RECENT_DAY_OPTIONS.find((option) => option.value === value)?.label ?? "最近 7 天";
}

export function filterSessionsByRecentDays(
  sessions: SessionData[],
  recentDays: RecentDaysFilter,
) {
  if (recentDays === "all") return sessions;
  const days = Number(recentDays);
  const cutoff = Date.now() - days * 24 * 60 * 60 * 1000;
  return sessions.filter((session) => {
    const activeAt = new Date(session.lastActiveAt).getTime();
    return Number.isNaN(activeAt) || activeAt >= cutoff;
  });
}

