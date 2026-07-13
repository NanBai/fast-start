import type { MouseEvent } from "react";
import { Icon } from "./icons/Icon";
import { formatRelative } from "../lib/sessionUtils";
import { CLI_LABELS, SessionData } from "../types";

export function SessionRow({
  session,
  active,
  launchingId,
  deletingId,
  showCliLabel = false,
  onLaunch,
  onContextMenu,
}: {
  session: SessionData;
  active: boolean;
  launchingId: string | null;
  deletingId: string | null;
  showCliLabel?: boolean;
  onLaunch: (sessionId: string) => Promise<void>;
  onContextMenu: (session: SessionData, event: MouseEvent<HTMLDivElement>) => void;
}) {
  const loading = launchingId === session.id;
  const deleting = deletingId === session.id;
  const busy = loading || deleting;
  const summary = session.summary?.trim() || null;
  return (
    <div
      className="session-row"
      data-active={active}
      data-busy={busy}
      onContextMenu={(event) => onContextMenu(session, event)}
    >
      <div className="session-main">
        {showCliLabel && (
          <span className="session-cli-label" data-cli={session.cliType}>
            {CLI_LABELS[session.cliType]}
          </span>
        )}
        <span className="session-name" title={summary ?? session.projectName}>
          {summary ?? session.projectName}
        </span>
        <span className="session-time">{formatRelative(session.lastActiveAt)}</span>
      </div>
      <button
        type="button"
        className="launch-btn"
        data-loading={loading || deleting}
        disabled={busy}
        onClick={() => void onLaunch(session.id)}
      >
        {loading || deleting ? (
          <>
            <Icon.Spinner /> {deleting ? "删除中" : "启动中"}
          </>
        ) : (
          <>
            启动 <Icon.Arrow />
          </>
        )}
      </button>
    </div>
  );
}
