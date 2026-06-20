import { useState } from "react";
import { Icon } from "./icons/Icon";
import { SessionRow } from "./SessionRow";
import { SessionData } from "../types";

export function ProjectBucket({
  projectDir,
  projectName,
  sessions,
  launchingId,
  onLaunch,
}: {
  projectDir: string;
  projectName: string;
  sessions: SessionData[];
  launchingId: string | null;
  onLaunch: (sessionId: string) => Promise<void>;
}) {
  const [expanded, setExpanded] = useState(true);

  return (
    <div className="project-bucket" data-open={expanded}>
      <button
        type="button"
        className="project-bucket-header"
        onClick={() => setExpanded((current) => !current)}
        aria-expanded={expanded}
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
      {expanded && (
        <div className="project-bucket-body">
          {sessions.map((session) => (
            <SessionRow
              key={session.id}
              session={session}
              launchingId={launchingId}
              onLaunch={onLaunch}
            />
          ))}
        </div>
      )}
    </div>
  );
}

