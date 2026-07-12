import { PortUsage } from "../types";

export function PortConfirmDialog({
  ports,
  closing,
  onCancel,
  onConfirm,
}: {
  ports: PortUsage[];
  closing: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const pidCount = new Set(ports.map((port) => port.pid)).size;
  const portText = ports.length === 1 ? `${ports[0].port}` : `${ports.length} 条端口记录`;

  return (
    <div className="confirm-backdrop" role="presentation">
      <section className="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="port-confirm-title">
        <h2 id="port-confirm-title">关闭端口服务？</h2>
        <p>
          将对 {portText} 对应的 {pidCount} 个当前用户进程发送 TERM 信号。关闭后会重新扫描端口列表。
        </p>
        <div className="confirm-actions">
          <button type="button" className="confirm-secondary" onClick={onCancel} disabled={closing}>
            取消
          </button>
          <button type="button" className="confirm-danger" onClick={onConfirm} disabled={closing}>
            {closing ? "关闭中…" : "关闭服务"}
          </button>
        </div>
      </section>
    </div>
  );
}
