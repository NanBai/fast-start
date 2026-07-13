import { CLI_LABELS, SessionData } from "../types";

export type RecentDaysFilter = "1" | "3" | "7" | "14" | "30" | "all";

export type ProjectSessionGroup = {
  projectDir: string;
  projectName: string;
  sessions: SessionData[];
};

export type QuickAccessOptions = {
  recentDays: RecentDaysFilter;
  query: string;
  favoriteProjectDirs: Set<string>;
  activeSessionId?: string | null;
};

export type QuickAccessResult = {
  sessions: SessionData[];
  activeSessionId: string | null;
  matchCount: number;
};

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
    if (Number.isNaN(activeAt)) {
      return false;
    }
    return activeAt >= cutoff;
  });
}

export function filterSessionsForQuickAccess(
  sessions: SessionData[],
  options: QuickAccessOptions,
): QuickAccessResult {
  const recentSessions = filterSessionsByRecentDays(sessions, options.recentDays);
  const query = normalizeQuery(options.query);
  const matchedSessions = query
    ? recentSessions.filter((session) => sessionMatchesQuery(session, query))
    : recentSessions;
  const sortedSessions = sortSessionsByFavoriteProject(
    matchedSessions,
    options.favoriteProjectDirs,
  );
  const activeSessionId =
    sortedSessions.find((session) => session.id === options.activeSessionId)?.id ??
    sortedSessions[0]?.id ??
    null;

  return {
    sessions: sortedSessions,
    activeSessionId,
    matchCount: sortedSessions.length,
  };
}

export function sanitizeFavoriteProjectDirs(
  projectDirs: string[],
  sessions: SessionData[],
) {
  const allowed = new Set(sessions.map((session) => session.projectDir));
  const seen = new Set<string>();
  return projectDirs.filter((projectDir) => {
    if (!allowed.has(projectDir) || seen.has(projectDir)) {
      return false;
    }
    seen.add(projectDir);
    return true;
  });
}

/**
 * 跨 CLI 按 projectDir 聚合；保留输入列表中的首次出现顺序
 * （调用方应先完成收藏排序 / 时间过滤）。
 */
export function groupSessionsByProject(sessions: SessionData[]): ProjectSessionGroup[] {
  const groups: ProjectSessionGroup[] = [];
  const groupIndex = new Map<string, number>();
  for (const session of sessions) {
    const existing = groupIndex.get(session.projectDir);
    if (existing === undefined) {
      groupIndex.set(session.projectDir, groups.length);
      groups.push({
        projectDir: session.projectDir,
        projectName: session.projectName,
        sessions: [session],
      });
    } else {
      groups[existing].sessions.push(session);
    }
  }
  return groups;
}

function sortSessionsByFavoriteProject(
  sessions: SessionData[],
  favoriteProjectDirs: Set<string>,
) {
  return sessions
    .map((session, index) => ({ session, index }))
    .sort((left, right) => {
      const leftFavorite = favoriteProjectDirs.has(left.session.projectDir);
      const rightFavorite = favoriteProjectDirs.has(right.session.projectDir);
      if (leftFavorite !== rightFavorite) {
        return leftFavorite ? -1 : 1;
      }
      return left.index - right.index;
    })
    .map((item) => item.session);
}

function sessionMatchesQuery(session: SessionData, normalizedQuery: string) {
  return [
    session.cliType,
    CLI_LABELS[session.cliType],
    session.projectName,
    session.projectDir,
    session.summary ?? "",
  ]
    .join(" ")
    .toLowerCase()
    .includes(normalizedQuery);
}

function normalizeQuery(query: string) {
  return query.trim().toLowerCase();
}
