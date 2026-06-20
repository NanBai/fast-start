import { Icon } from "./icons/Icon";
import { formatRelative } from "../lib/sessionUtils";
import { SessionData } from "../types";

export function SessionRow({
  session,
  launchingId,
  onLaunch,
}: {
  session: SessionData;
  launchingId: string | null;
  onLaunch: (sessionId: string) => Promise<void>;
}) {
  const loading = launchingId === session.id;
  const summary = session.summary?.trim() || null;
  return (
    <div className="session-row">
      <div className="session-main">
        <span className="session-name" title={summary ?? session.projectName}>
          {summary ?? session.projectName}
        </span>
        <span className="session-time">{formatRelative(session.lastActiveAt)}</span>
      </div>
      <button
        type="button"
        className="launch-btn"
        data-loading={loading}
        disabled={loading}
        onClick={() => void onLaunch(session.id)}
      >
        {loading ? (
          <>
            <Icon.Spinner /> 启动中
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

