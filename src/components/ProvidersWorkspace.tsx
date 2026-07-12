import { FormEvent, useMemo, useState } from "react";
import { emptyGrokProfile, GrokBackupInfo, GrokProfile, GrokProviderStatus } from "../types";

function maskKey(key: string) {
  if (!key) return "（未设置）";
  if (key.length <= 6) return "••••";
  return `${key.slice(0, 3)}…${key.slice(-2)}`;
}

export function ProvidersWorkspace({
  profiles,
  status,
  backups,
  loading,
  busyId,
  onRefresh,
  onActivate,
  onImport,
  onSave,
  onDelete,
  onRestore,
}: {
  profiles: GrokProfile[];
  status: GrokProviderStatus | null;
  backups: GrokBackupInfo[];
  loading: boolean;
  busyId: string | null;
  onRefresh: () => void;
  onActivate: (id: string) => void;
  onImport: () => void;
  onSave: (profile: GrokProfile, activateAfter: boolean) => Promise<GrokProfile | null>;
  onDelete: (id: string) => void;
  onRestore: (file: string) => void;
}) {
  const [editing, setEditing] = useState<GrokProfile | null>(null);
  const [query, setQuery] = useState("");

  const visible = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return profiles;
    return profiles.filter(
      (p) =>
        p.name.toLowerCase().includes(q) ||
        p.baseUrl.toLowerCase().includes(q) ||
        p.defaultModel.toLowerCase().includes(q),
    );
  }, [profiles, query]);

  async function handleSubmit(event: FormEvent, activateAfter: boolean) {
    event.preventDefault();
    if (!editing) return;
    const draft = {
      ...editing,
      models:
        editing.models.length > 0
          ? editing.models
          : editing.defaultModel
            ? [
                {
                  name: editing.defaultModel,
                  model: editing.defaultModel,
                  baseUrl: editing.baseUrl,
                  apiKey: editing.apiKey,
                  apiBackend: "chat_completions",
                  extraHeaders: {},
                  supportsBackendSearch: false,
                  contextWindow: 0,
                  maxCompletionTokens: 0,
                },
              ]
            : [],
    };
    const saved = await onSave(draft, activateAfter);
    if (saved) setEditing(null);
  }

  return (
    <section className="providers-workspace">
      {status && !status.configMatchesActive && status.activeProfile && (
        <div className="providers-banner" role="status">
          <div>
            <strong>配置与当前供应商不一致</strong>
            <p>
              `config.toml` 与已启用的「{status.activeProfile.name}」不匹配。可重新启用该供应商覆盖文件。
            </p>
          </div>
          {status.activeProfile && (
            <button
              type="button"
              className="btn primary sm"
              disabled={busyId === status.activeProfile.id}
              onClick={() => onActivate(status.activeProfile!.id)}
            >
              重新启用
            </button>
          )}
        </div>
      )}

      <div className="providers-toolbar">
        <div>
          <h2 className="providers-title">Grok 供应商</h2>
          <p className="providers-desc">
            切换 `~/.grok/config.toml` 上游；新开 Grok 会话生效
          </p>
        </div>
        <div className="providers-actions">
          <button type="button" className="btn" onClick={() => onRefresh()} disabled={loading}>
            刷新
          </button>
          <button type="button" className="btn" onClick={() => onImport()} disabled={!!busyId}>
            导入当前配置
          </button>
          <button
            type="button"
            className="btn primary"
            onClick={() => setEditing(emptyGrokProfile())}
          >
            添加供应商
          </button>
        </div>
      </div>

      <input
        className="providers-search"
        type="search"
        placeholder="搜索供应商名称、URL 或模型…"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />

      {status && (
        <p className="providers-meta muted">
          配置：{status.configPath}
          {status.configExists ? "" : "（文件不存在）"} · 档案目录：{status.dataDir}
        </p>
      )}

      {loading && profiles.length === 0 ? (
        <div className="empty-state">加载中…</div>
      ) : visible.length === 0 ? (
        <div className="empty-state">
          <h3>还没有供应商</h3>
          <p>从当前 config.toml 导入，或添加一个新的上游配置。</p>
        </div>
      ) : (
        <div className="provider-grid">
          {visible.map((profile) => (
            <article
              key={profile.id}
              className="provider-card"
              data-active={profile.isActive}
            >
              <header className="provider-card-head">
                <div>
                  <h3>{profile.name}</h3>
                  <p className="muted mono">{profile.baseUrl || "（无 Base URL）"}</p>
                </div>
                {profile.isActive && <span className="provider-badge">启用中</span>}
              </header>
              <dl className="provider-fields">
                <div>
                  <dt>默认模型</dt>
                  <dd>{profile.defaultModel || "—"}</dd>
                </div>
                <div>
                  <dt>API Key</dt>
                  <dd className="mono">{maskKey(profile.apiKey)}</dd>
                </div>
              </dl>
              <footer className="provider-card-actions">
                <button
                  type="button"
                  className="btn sm primary"
                  disabled={profile.isActive || busyId === profile.id}
                  onClick={() => onActivate(profile.id)}
                >
                  {profile.isActive ? "已启用" : "启用"}
                </button>
                <button
                  type="button"
                  className="btn sm"
                  onClick={() => setEditing({ ...profile })}
                >
                  编辑
                </button>
                <button
                  type="button"
                  className="btn sm danger"
                  disabled={busyId === profile.id}
                  onClick={() => {
                    if (confirm(`删除供应商「${profile.name}」？不会删除 config.toml`)) {
                      onDelete(profile.id);
                    }
                  }}
                >
                  删除
                </button>
              </footer>
            </article>
          ))}
        </div>
      )}

      <section className="providers-backups">
        <h3>配置备份</h3>
        {backups.length === 0 ? (
          <p className="muted">暂无备份（启用供应商时会自动生成）</p>
        ) : (
          <ul className="backup-list">
            {backups.slice(0, 8).map((b) => (
              <li key={b.file}>
                <span className="mono">{b.file}</span>
                <span className="muted">
                  {new Date(b.createdAt).toLocaleString()} · {Math.max(1, Math.round(b.size / 1024))}{" "}
                  KB
                </span>
                <button
                  type="button"
                  className="btn sm"
                  disabled={busyId === b.file}
                  onClick={() => {
                    if (confirm(`用备份 ${b.file} 覆盖当前 config.toml？`)) {
                      onRestore(b.file);
                    }
                  }}
                >
                  还原
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      {editing && (
        <div className="providers-modal" role="dialog" aria-modal="true">
          <form
            className="providers-form"
            onSubmit={(e) => void handleSubmit(e, false)}
          >
            <h3>{editing.id ? "编辑供应商" : "添加供应商"}</h3>
            <label>
              名称
              <input
                required
                value={editing.name}
                onChange={(e) => setEditing({ ...editing, name: e.target.value })}
              />
            </label>
            <label>
              Base URL
              <input
                value={editing.baseUrl}
                placeholder="https://api.example.com/v1"
                onChange={(e) => setEditing({ ...editing, baseUrl: e.target.value })}
              />
            </label>
            <label>
              API Key
              <input
                type="password"
                autoComplete="off"
                value={editing.apiKey}
                onChange={(e) => setEditing({ ...editing, apiKey: e.target.value })}
              />
            </label>
            <label>
              默认模型
              <input
                value={editing.defaultModel}
                onChange={(e) =>
                  setEditing({
                    ...editing,
                    defaultModel: e.target.value,
                    webSearchModel: editing.webSearchModel || e.target.value,
                    subagentsDefaultModel: editing.subagentsDefaultModel || e.target.value,
                  })
                }
              />
            </label>
            <label>
              联网搜索模型
              <input
                value={editing.webSearchModel}
                onChange={(e) => setEditing({ ...editing, webSearchModel: e.target.value })}
              />
            </label>
            <label>
              Subagents 模型
              <input
                value={editing.subagentsDefaultModel}
                onChange={(e) =>
                  setEditing({ ...editing, subagentsDefaultModel: e.target.value })
                }
              />
            </label>
            <div className="providers-form-actions">
              <button type="button" className="btn" onClick={() => setEditing(null)}>
                取消
              </button>
              <button type="submit" className="btn" disabled={!!busyId}>
                仅保存
              </button>
              <button
                type="button"
                className="btn primary"
                disabled={!!busyId}
                onClick={(e) => void handleSubmit(e as unknown as FormEvent, true)}
              >
                保存并启用
              </button>
            </div>
          </form>
        </div>
      )}
    </section>
  );
}
