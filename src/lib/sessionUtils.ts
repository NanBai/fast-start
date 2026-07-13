import {
  CLI_LABELS,
  SessionData,
  SessionHealth,
  SessionHealthFilter,
} from "../types";

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
  favoriteSessionIds?: Set<string>;
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

const SHORT_ID_MIN = 7;
const SHORT_ID_MAX = 12;

/** 列表展示用的短 id：取 native sessionId 末尾 N 位可打印字符 */
export function shortSessionId(sessionId: string, length = SHORT_ID_MIN): string {
  const clean = sessionId.replace(/[^a-zA-Z0-9]/g, "");
  const n = Math.max(1, Math.min(length, SHORT_ID_MAX));
  if (!clean) return sessionId.slice(0, n);
  if (clean.length <= n) return clean;
  return clean.slice(-n);
}

/**
 * 在同一批 session 内生成互不撞车的短 id（session.sessionId → short）。
 * 从 7 位起加长，直到组内唯一；仍撞车则加序号后缀。
 */
export function uniqueShortSessionIds(
  sessions: SessionData[],
): Map<string, string> {
  const result = new Map<string, string>();
  if (sessions.length === 0) return result;

  for (let length = SHORT_ID_MIN; length <= SHORT_ID_MAX; length += 1) {
    const used = new Map<string, string>();
    const attempt = new Map<string, string>();
    let collision = false;
    for (const session of sessions) {
      const short = shortSessionId(session.sessionId, length);
      const owner = used.get(short);
      if (owner !== undefined && owner !== session.sessionId) {
        collision = true;
        break;
      }
      used.set(short, session.sessionId);
      attempt.set(session.sessionId, short);
    }
    if (!collision) return attempt;
  }

  // 极端回退：同前缀加序号，保证组内唯一
  const seen = new Map<string, number>();
  for (const session of sessions) {
    const base = shortSessionId(session.sessionId, SHORT_ID_MAX);
    const n = (seen.get(base) ?? 0) + 1;
    seen.set(base, n);
    result.set(session.sessionId, n === 1 ? base : `${base}${n}`);
  }
  return result;
}

/** 具体时刻，便于同简介 session 区分（相对时间太粗） */
export function formatSessionClock(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "";
  const now = new Date();
  const hh = String(date.getHours()).padStart(2, "0");
  const mm = String(date.getMinutes()).padStart(2, "0");
  const time = `${hh}:${mm}`;
  const sameDay =
    date.getFullYear() === now.getFullYear() &&
    date.getMonth() === now.getMonth() &&
    date.getDate() === now.getDate();
  if (sameDay) return time;
  const month = date.getMonth() + 1;
  const day = date.getDate();
  if (date.getFullYear() === now.getFullYear()) {
    return `${month}/${day} ${time}`;
  }
  return `${date.getFullYear()}/${month}/${day} ${time}`;
}

export function sessionTitle(session: SessionData): string {
  const summary = session.summary?.trim();
  return summary && summary.length > 0 ? summary : session.projectName;
}

/**
 * 同一批 session 里标题重复的 key 集合（小写）。
 * 用于列表副行强化短 id，避免同名简介无法区分。
 */
export function ambiguousSessionTitleKeys(sessions: SessionData[]): Set<string> {
  const counts = new Map<string, number>();
  for (const session of sessions) {
    const key = sessionTitle(session).toLowerCase();
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  const ambiguous = new Set<string>();
  for (const [key, count] of counts) {
    if (count > 1) ambiguous.add(key);
  }
  return ambiguous;
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

/**
 * 最后活跃早于「现在 − days 天」的 session（用于批量勾选清理）。
 * 无效时间戳的条目不入选。
 */
export function sessionsOlderThanDays(
  sessions: SessionData[],
  days: number,
): SessionData[] {
  if (days <= 0) return [];
  const cutoff = Date.now() - days * 24 * 60 * 60 * 1000;
  return sessions.filter((session) => {
    const activeAt = new Date(session.lastActiveAt).getTime();
    if (Number.isNaN(activeAt)) {
      return false;
    }
    return activeAt < cutoff;
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
  const sortedSessions = sortSessionsByFavorites(
    matchedSessions,
    options.favoriteSessionIds ?? new Set(),
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

/** 基于 inspect_session_health 报告筛选陈旧 session（不读路径）。 */
export function filterSessionsByHealth(
  sessions: SessionData[],
  healthById: Map<string, SessionHealth>,
  filter: SessionHealthFilter,
): SessionData[] {
  if (filter === "all") return sessions;
  return sessions.filter((session) => {
    const health = healthById.get(session.id);
    if (!health) return false;
    if (filter === "missing_cwd") {
      return health.flags.includes("missing_cwd") || !health.cwdExists;
    }
    if (filter === "missing_source") {
      return health.flags.includes("missing_source") || health.sourceExists === false;
    }
    // stale：缺 cwd 或缺源
    return (
      health.flags.includes("missing_cwd") ||
      health.flags.includes("missing_source") ||
      !health.cwdExists ||
      health.sourceExists === false
    );
  });
}

export function sessionHealthBadge(health: SessionHealth | undefined): string | null {
  if (!health) return null;
  const parts: string[] = [];
  if (health.flags.includes("missing_cwd") || !health.cwdExists) parts.push("缺目录");
  if (health.flags.includes("missing_source") || health.sourceExists === false) {
    parts.push("缺源");
  }
  if (parts.length === 0) return null;
  return parts.join("·");
}

export type DiskUsageBucket = {
  key: string;
  label: string;
  bytes: number | null;
  knownCount: number;
  unknownCount: number;
  sizeCapped: boolean;
};

/** 按 CLI / 项目聚合 inspect 的 approxBytes；null 不计入合计。 */
export function aggregateDiskUsage(
  sessions: SessionData[],
  healthById: Map<string, SessionHealth>,
  mode: "cli" | "project",
): DiskUsageBucket[] {
  type Acc = {
    label: string;
    bytes: number;
    knownCount: number;
    unknownCount: number;
    sizeCapped: boolean;
  };
  const map = new Map<string, Acc>();
  for (const session of sessions) {
    const key = mode === "cli" ? session.cliType : session.projectDir;
    const label =
      mode === "cli" ? CLI_LABELS[session.cliType] : session.projectName || session.projectDir;
    const acc = map.get(key) ?? {
      label,
      bytes: 0,
      knownCount: 0,
      unknownCount: 0,
      sizeCapped: false,
    };
    const health = healthById.get(session.id);
    if (health?.flags.includes("size_capped")) acc.sizeCapped = true;
    if (health && typeof health.approxBytes === "number") {
      acc.bytes += health.approxBytes;
      acc.knownCount += 1;
    } else {
      acc.unknownCount += 1;
    }
    map.set(key, acc);
  }
  return Array.from(map.entries())
    .map(([key, acc]) => ({
      key,
      label: acc.label,
      bytes: acc.knownCount > 0 ? acc.bytes : null,
      knownCount: acc.knownCount,
      unknownCount: acc.unknownCount,
      sizeCapped: acc.sizeCapped,
    }))
    .sort((a, b) => (b.bytes ?? -1) - (a.bytes ?? -1));
}

export function formatBytes(bytes: number | null): string {
  if (bytes == null) return "未知";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
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

export function sanitizeFavoriteSessionIds(
  sessionIds: string[],
  sessions: SessionData[],
) {
  const allowed = new Set(sessions.map((session) => session.id));
  const seen = new Set<string>();
  return sessionIds.filter((id) => {
    if (!allowed.has(id) || seen.has(id)) {
      return false;
    }
    seen.add(id);
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

/** 排序：session 收藏 > 项目收藏 > 原时间序（稳定 index） */
function sortSessionsByFavorites(
  sessions: SessionData[],
  favoriteSessionIds: Set<string>,
  favoriteProjectDirs: Set<string>,
) {
  return sessions
    .map((session, index) => ({ session, index }))
    .sort((left, right) => {
      const leftSessionFav = favoriteSessionIds.has(left.session.id);
      const rightSessionFav = favoriteSessionIds.has(right.session.id);
      if (leftSessionFav !== rightSessionFav) {
        return leftSessionFav ? -1 : 1;
      }
      const leftProjectFav = favoriteProjectDirs.has(left.session.projectDir);
      const rightProjectFav = favoriteProjectDirs.has(right.session.projectDir);
      if (leftProjectFav !== rightProjectFav) {
        return leftProjectFav ? -1 : 1;
      }
      return left.index - right.index;
    })
    .map((item) => item.session);
}

function sessionMatchesQuery(session: SessionData, normalizedQuery: string) {
  const shortId = shortSessionId(session.sessionId);
  return [
    session.cliType,
    CLI_LABELS[session.cliType],
    session.projectName,
    session.projectDir,
    session.summary ?? "",
    session.sessionId,
    shortId,
    `#${shortId}`,
  ]
    .join(" ")
    .toLowerCase()
    .includes(normalizedQuery);
}

function normalizeQuery(query: string) {
  // 支持粘贴 UI 上的 `#a1b2c3d` 短 id
  return query.trim().toLowerCase().replace(/^#+/, "");
}
