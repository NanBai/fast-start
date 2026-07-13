/**
 * 轻量 node 断言（无 vitest）：pnpm exec node src/lib/sessionUtils.bulkSelect.test.mjs
 * 逻辑与 sessionUtils.ts 中 sessionsOlderThanDays / takeBulkSelectIds 保持同步。
 */

const BULK_SELECT_LIMIT = 50;

function sessionsOlderThanDays(sessions, days, nowMs = Date.now()) {
  if (days <= 0) return [];
  const cutoff = nowMs - days * 24 * 60 * 60 * 1000;
  return sessions
    .filter((session) => {
      const activeAt = new Date(session.lastActiveAt).getTime();
      if (Number.isNaN(activeAt)) return false;
      return activeAt < cutoff;
    })
    .sort(
      (a, b) =>
        new Date(a.lastActiveAt).getTime() - new Date(b.lastActiveAt).getTime(),
    );
}

function takeBulkSelectIds(ids, limit = BULK_SELECT_LIMIT) {
  const unique = Array.from(new Set(ids.filter(Boolean)));
  if (unique.length <= limit) {
    return { selected: unique, total: unique.length, truncated: false };
  }
  return {
    selected: unique.slice(0, limit),
    total: unique.length,
    truncated: true,
  };
}

const day = 24 * 60 * 60 * 1000;
const now = Date.parse("2026-07-13T12:00:00.000Z");

function assert(cond, msg) {
  if (!cond) throw new Error(msg);
}

// sessionsOlderThanDays
{
  const sessions = [
    { id: "a", lastActiveAt: new Date(now - 3 * day).toISOString() },
    { id: "b", lastActiveAt: new Date(now - 10 * day).toISOString() },
    { id: "c", lastActiveAt: new Date(now - 40 * day).toISOString() },
    { id: "bad", lastActiveAt: "not-a-date" },
  ];
  const older7 = sessionsOlderThanDays(sessions, 7, now);
  assert(older7.map((s) => s.id).join(",") === "c,b", "older7 order oldest-first");
  assert(sessionsOlderThanDays(sessions, 0, now).length === 0, "days<=0 empty");
  const older30 = sessionsOlderThanDays(sessions, 30, now);
  assert(older30.map((s) => s.id).join(",") === "c", "only c older than 30d");
}

// takeBulkSelectIds
{
  const ids = Array.from({ length: 60 }, (_, i) => `id-${i}`);
  const r = takeBulkSelectIds(ids, 50);
  assert(r.truncated && r.total === 60 && r.selected.length === 50, "truncate 50");
  assert(r.selected[0] === "id-0" && r.selected[49] === "id-49", "keep head");
  const d = takeBulkSelectIds(["a", "a", "", "b"], 50);
  assert(d.selected.join(",") === "a,b" && !d.truncated, "dedupe");
}

console.log("sessionUtils.bulkSelect.test.mjs: ok");
