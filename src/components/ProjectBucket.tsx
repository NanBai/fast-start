import type { MouseEvent } from "react";
import { useMemo, useState } from "react";
import { Icon } from "./icons/Icon";
import { SessionRow } from "./SessionRow";
import {
  ambiguousSessionTitleKeys,
  sessionTitle,
  uniqueShortSessionIds,
} from "../lib/sessionUtils";
import { SessionData } from "../types";

export function ProjectBucket({
  projectDir,
  projectName,
  sessions,
  favorite,
  forceOpen = false,
  showCliLabel = false,
  favoriteSessionIds,
  activeSessionId,
  launchingId,
  deletingId,
  healthBadgeFor,
  onLaunch,
  onToggleFavorite,
  onToggleSessionFavorite,
  onSessionContextMenu,
}: {
  projectDir: string;
  projectName: string;
  sessions: SessionData[];
  favorite: boolean;
  forceOpen?: boolean;
  showCliLabel?: boolean;
  favoriteSessionIds: Set<string>;
  activeSessionId: string | null;
  launchingId: string | null;
  deletingId: string | null;
  healthBadgeFor?: (sessionId: string) => string | null;
  onLaunch: (sessionId: string) => Promise<void>;
  onToggleFavorite: (projectDir: string) => void;
  onToggleSessionFavorite: (sessionId: string) => void;
  onSessionContextMenu: (
    session: SessionData,
    event: MouseEvent<HTMLDivElement>,
  ) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const open = forceOpen || expanded;
  const panelId = `project-bucket-${projectDir.replace(/[^a-zA-Z0-9_-]/g, "_")}`;
  const ambiguousTitles = useMemo(
    () => ambiguousSessionTitleKeys(sessions),
    [sessions],
  );
  const shortIds = useMemo(() => uniqueShortSessionIds(sessions), [sessions]);

  return (
    <div className="project-bucket" data-open={open} data-favorite={favorite}>
      <div className="project-bucket-header">
        <button
          type="button"
          className="project-bucket-toggle"
          onClick={() => setExpanded((current) => !current)}
          aria-expanded={open}
          aria-controls={panelId}
        >
          <span className="project-folder" aria-hidden="true">
            <Icon.Folder />
          </span>
          <span className="project-title">
            <span className="project-name" title={projectDir}>{projectName}</span>
            <span className="project-path" title={projectDir}>{projectDir}</span>
          </span>
          <span className="project-session-count">{sessions.length}</span>
          <span className="project-chev" aria-hidden="true">
            <Icon.Chevron />
          </span>
        </button>
        <button
          type="button"
          className="project-favorite-btn"
          data-active={favorite}
          aria-label={favorite ? "取消收藏项目" : "收藏项目"}
          title={favorite ? "取消收藏项目" : "收藏项目"}
          onClick={() => onToggleFavorite(projectDir)}
        >
          <Icon.Star />
        </button>
      </div>
      <div
        id={panelId}
        className="project-bucket-collapse"
        role="region"
        aria-label={`${projectName} sessions`}
        aria-hidden={!open}
        inert={!open ? true : undefined}
      >
        <div className="project-bucket-collapse-inner">
          <div className="project-bucket-body">
            {sessions.map((session) => (
              <SessionRow
                key={session.id}
                session={session}
                active={activeSessionId === session.id}
                launchingId={launchingId}
                deletingId={deletingId}
                showCliLabel={showCliLabel}
                favorite={favoriteSessionIds.has(session.id)}
                titleAmbiguous={ambiguousTitles.has(
                  sessionTitle(session).toLowerCase(),
                )}
                displayShortId={shortIds.get(session.sessionId)}
                healthBadge={healthBadgeFor?.(session.id) ?? null}
                onLaunch={onLaunch}
                onToggleFavorite={onToggleSessionFavorite}
                onContextMenu={onSessionContextMenu}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
