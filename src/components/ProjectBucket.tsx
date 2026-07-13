import type { MouseEvent } from "react";
import { useState } from "react";
import { Icon } from "./icons/Icon";
import { SessionRow } from "./SessionRow";
import { SessionData } from "../types";

export function ProjectBucket({
  projectDir,
  projectName,
  sessions,
  favorite,
  forceOpen = false,
  showCliLabel = false,
  activeSessionId,
  launchingId,
  deletingId,
  onLaunch,
  onToggleFavorite,
  onSessionContextMenu,
}: {
  projectDir: string;
  projectName: string;
  sessions: SessionData[];
  favorite: boolean;
  forceOpen?: boolean;
  showCliLabel?: boolean;
  activeSessionId: string | null;
  launchingId: string | null;
  deletingId: string | null;
  onLaunch: (sessionId: string) => Promise<void>;
  onToggleFavorite: (projectDir: string) => void;
  onSessionContextMenu: (
    session: SessionData,
    event: MouseEvent<HTMLDivElement>,
  ) => void;
}) {
  const [expanded, setExpanded] = useState(true);
  const open = forceOpen || expanded;

  return (
    <div className="project-bucket" data-open={open} data-favorite={favorite}>
      <div className="project-bucket-header">
        <button
          type="button"
          className="project-bucket-toggle"
          onClick={() => setExpanded((current) => !current)}
          aria-expanded={open}
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
      {open && (
        <div className="project-bucket-body">
          {sessions.map((session) => (
            <SessionRow
              key={session.id}
              session={session}
              active={activeSessionId === session.id}
              launchingId={launchingId}
              deletingId={deletingId}
              showCliLabel={showCliLabel}
              onLaunch={onLaunch}
              onContextMenu={onSessionContextMenu}
            />
          ))}
        </div>
      )}
    </div>
  );
}
