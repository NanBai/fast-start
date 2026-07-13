import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type MouseEvent,
  type UIEvent,
} from "react";
import { SessionRow } from "./SessionRow";
import {
  ambiguousSessionTitleKeys,
  sessionTitle,
  uniqueShortSessionIds,
} from "../lib/sessionUtils";
import { computeVirtualWindow } from "../lib/sessionListVirtual";
import type { SessionData } from "../types";

/** 与 session-row 视觉高度大致对齐（含 padding）。 */
const ROW_HEIGHT = 52;
const VIEWPORT_MAX = 360;
/** 低于此数量不虚拟化，避免小列表 overhead。 */
const VIRTUALIZE_THRESHOLD = 24;

export function VirtualSessionRows({
  sessions,
  showCliLabel = false,
  favoriteSessionIds,
  activeSessionId,
  launchingId,
  deletingId,
  healthBadgeFor,
  selectedIds,
  onToggleSelected,
  onLaunch,
  onToggleSessionFavorite,
  onSessionContextMenu,
}: {
  sessions: SessionData[];
  showCliLabel?: boolean;
  favoriteSessionIds: Set<string>;
  activeSessionId: string | null;
  launchingId: string | null;
  deletingId: string | null;
  healthBadgeFor?: (sessionId: string) => string | null;
  selectedIds?: Set<string>;
  onToggleSelected?: (sessionId: string) => void;
  onLaunch: (sessionId: string) => Promise<void>;
  onToggleSessionFavorite: (sessionId: string) => void;
  onSessionContextMenu: (
    session: SessionData,
    event: MouseEvent<HTMLDivElement>,
  ) => void;
}) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const ambiguousTitles = useMemo(
    () => ambiguousSessionTitleKeys(sessions),
    [sessions],
  );
  const shortIds = useMemo(() => uniqueShortSessionIds(sessions), [sessions]);

  const useVirtual = sessions.length >= VIRTUALIZE_THRESHOLD;
  const viewportHeight = Math.min(
    VIEWPORT_MAX,
    Math.max(ROW_HEIGHT * 4, sessions.length * ROW_HEIGHT),
  );
  const window = useVirtual
    ? computeVirtualWindow(scrollTop, viewportHeight, sessions.length, ROW_HEIGHT)
    : {
        start: 0,
        end: sessions.length,
        offsetTop: 0,
        totalHeight: sessions.length * ROW_HEIGHT,
      };

  const slice = sessions.slice(window.start, window.end);

  // 键盘导航：活跃行滚入可视区（虚拟列表改 scrollTop；小列表用 scrollIntoView）
  useEffect(() => {
    if (!activeSessionId) return;
    const index = sessions.findIndex((s) => s.id === activeSessionId);
    if (index < 0) return;

    if (useVirtual) {
      const el = scrollRef.current;
      if (!el) return;
      const rowTop = index * ROW_HEIGHT;
      const rowBottom = rowTop + ROW_HEIGHT;
      const viewTop = el.scrollTop;
      const viewBottom = viewTop + el.clientHeight;
      if (rowTop < viewTop) {
        el.scrollTop = rowTop;
      } else if (rowBottom > viewBottom) {
        el.scrollTop = Math.max(0, rowBottom - el.clientHeight);
      }
      return;
    }

    const node = document.querySelector<HTMLElement>(
      `[data-session-list-id="${CSS.escape(activeSessionId)}"]`,
    );
    node?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }, [activeSessionId, sessions, useVirtual]);

  function onScroll(event: UIEvent<HTMLDivElement>) {
    setScrollTop(event.currentTarget.scrollTop);
  }

  function renderRow(session: SessionData) {
    return (
      <SessionRow
        key={session.id}
        session={session}
        active={activeSessionId === session.id}
        launchingId={launchingId}
        deletingId={deletingId}
        showCliLabel={showCliLabel}
        favorite={favoriteSessionIds.has(session.id)}
        titleAmbiguous={ambiguousTitles.has(sessionTitle(session).toLowerCase())}
        displayShortId={shortIds.get(session.sessionId)}
        healthBadge={healthBadgeFor?.(session.id) ?? null}
        selected={selectedIds?.has(session.id) ?? false}
        onToggleSelected={onToggleSelected}
        onLaunch={onLaunch}
        onToggleFavorite={onToggleSessionFavorite}
        onContextMenu={onSessionContextMenu}
      />
    );
  }

  if (!useVirtual) {
    return <div className="project-bucket-body">{sessions.map(renderRow)}</div>;
  }

  return (
    <div
      ref={scrollRef}
      className="project-bucket-body project-bucket-body-virtual"
      style={{ maxHeight: viewportHeight, overflowY: "auto" }}
      onScroll={onScroll}
    >
      <div style={{ height: window.totalHeight, position: "relative" }}>
        <div style={{ transform: `translateY(${window.offsetTop}px)` }}>
          {slice.map(renderRow)}
        </div>
      </div>
    </div>
  );
}
