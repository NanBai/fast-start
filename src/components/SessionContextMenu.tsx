import { Icon } from "./icons/Icon";
import { SessionData } from "../types";

export function SessionContextMenu({
  session,
  x,
  y,
  disabled,
  onDelete,
}: {
  session: SessionData;
  x: number;
  y: number;
  disabled: boolean;
  onDelete: (session: SessionData) => void;
}) {
  return (
    <div
      className="session-context-menu"
      style={{ left: x, top: y }}
      role="menu"
      aria-label="session 操作"
    >
      <button
        type="button"
        role="menuitem"
        className="session-context-item"
        disabled={disabled}
        onClick={() => onDelete(session)}
      >
        <Icon.Trash />
        删除此 session
      </button>
    </div>
  );
}
