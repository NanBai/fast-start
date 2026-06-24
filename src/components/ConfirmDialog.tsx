import { SessionData } from "../types";

export function ConfirmDialog({
  session,
  deleting,
  onCancel,
  onConfirm,
}: {
  session: SessionData;
  deleting: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const title = session.summary?.trim() || session.projectName;

  return (
    <div className="dialog-backdrop" role="presentation">
      <section
        className="confirm-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="delete-session-title"
      >
        <div className="confirm-copy">
          <h2 id="delete-session-title">删除此 session？</h2>
          <p title={title}>{title}</p>
        </div>
        <div className="confirm-actions">
          <button
            type="button"
            className="confirm-btn"
            disabled={deleting}
            onClick={onCancel}
          >
            取消
          </button>
          <button
            type="button"
            className="confirm-btn danger"
            disabled={deleting}
            onClick={onConfirm}
          >
            {deleting ? "删除中" : "删除"}
          </button>
        </div>
      </section>
    </div>
  );
}
