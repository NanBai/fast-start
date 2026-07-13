import type { MouseEvent } from "react";
import { BrandMark } from "./icons/BrandMark";
import { Icon } from "./icons/Icon";
import {
  formatRelative,
  formatSessionClock,
  shortSessionId,
  sessionTitle,
} from "../lib/sessionUtils";
import { CLI_LABELS, SessionData } from "../types";

export function SessionRow({
  session,
  active,
  launchingId,
  deletingId,
  showCliLabel = false,
  favorite = false,
  titleAmbiguous = false,
  displayShortId,
  healthBadge = null,
  onLaunch,
  onToggleFavorite,
  onContextMenu,
}: {
  session: SessionData;
  active: boolean;
  launchingId: string | null;
  deletingId: string | null;
  showCliLabel?: boolean;
  favorite?: boolean;
  /** 同项目内标题重复时，副行短 id 更醒目 */
  titleAmbiguous?: boolean;
  /** 组内已消歧的短 id；缺省时回退 shortSessionId */
  displayShortId?: string;
  /** 健康探测角标（缺目录/缺源），不含路径 */
  healthBadge?: string | null;
  onLaunch: (sessionId: string) => Promise<void>;
  onToggleFavorite?: (sessionId: string) => void;
  onContextMenu: (session: SessionData, event: MouseEvent<HTMLDivElement>) => void;
}) {
  const loading = launchingId === session.id;
  const deleting = deletingId === session.id;
  const busy = loading || deleting;
  const title = sessionTitle(session);
  const shortId = displayShortId ?? shortSessionId(session.sessionId);
  const clock = formatSessionClock(session.lastActiveAt);
  const tooltip = [title, session.sessionId, session.lastActiveAt]
    .filter(Boolean)
    .join("\n");

  return (
    <div
      className="session-row"
      data-active={active}
      data-busy={busy}
      data-favorite={favorite}
      data-ambiguous={titleAmbiguous}
      onContextMenu={(event) => onContextMenu(session, event)}
    >
      <div className="session-main">
        {showCliLabel && (
          <span
            className="session-cli-label"
            data-cli={session.cliType}
            title={CLI_LABELS[session.cliType]}
          >
            <span className="session-cli-mark" aria-hidden="true">
              <BrandMark cliType={session.cliType} />
            </span>
            <span className="session-cli-text">{CLI_LABELS[session.cliType]}</span>
          </span>
        )}
        <div className="session-text" title={tooltip}>
          <span className="session-name">{title}</span>
          <span className="session-submeta" data-visible={titleAmbiguous ? "always" : "hover"}>
            <span className="session-id" data-emphasize={titleAmbiguous}>
              #{shortId}
            </span>
            {clock && <span className="session-clock">{clock}</span>}
            {healthBadge && (
              <span className="session-health-badge" title={healthBadge}>
                {healthBadge}
              </span>
            )}
          </span>
        </div>
        <span className="session-time">{formatRelative(session.lastActiveAt)}</span>
      </div>
      <div className="session-actions">
        {onToggleFavorite && (
          <button
            type="button"
            className="session-favorite-btn"
            data-active={favorite}
            aria-label={favorite ? "取消收藏 session" : "收藏 session"}
            title={favorite ? "取消收藏 session" : "收藏 session"}
            onClick={() => onToggleFavorite(session.id)}
          >
            <Icon.Star />
          </button>
        )}
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
    </div>
  );
}
