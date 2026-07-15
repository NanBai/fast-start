import { FormEvent, useMemo, useState } from "react";
import {
  emptyGrokProfile,
  GrokBackupInfo,
  GrokHealthReport,
  GrokProfile,
  GrokProviderLayout,
  GrokProviderStatus,
} from "../types";
import {
  buildProviderCards,
  filterProviderCards,
  OFFICIAL_PROVIDER_KEY,
  type ProviderCard,
} from "../lib/grokProviderCards";

function maskKey(key: string) {
  if (!key) return "（未设置）";
  if (key.length <= 6) return "••••";
  return `${key.slice(0, 3)}…${key.slice(-2)}`;
}

export function ProvidersWorkspace({
  profiles,
  status,
  backups,
  health,
  layout,
  loading,
  busyId,
  onRefresh,
  onActivate,
  onActivateOfficial,
  onApplyPrivacy,
  onSaveLayout,
  onImport,
  onSave,
  onDelete,
  onRestore,
  onFetchModels,
  onTestConnection,
  onPreviewApply,
}: {
  profiles: GrokProfile[];
  status: GrokProviderStatus | null;
  backups: GrokBackupInfo[];
  health: GrokHealthReport | null;
  layout: GrokProviderLayout;
  loading: boolean;
  busyId: string | null;
  onRefresh: () => void;
  onActivate: (id: string) => void;
  onActivateOfficial: () => void;
  onApplyPrivacy: () => void;
  onSaveLayout: (layout: GrokProviderLayout) => Promise<GrokProviderLayout | null>;
  onImport: () => void;
  onSave: (profile: GrokProfile, activateAfter: boolean) => Promise<GrokProfile | null>;
  onDelete: (id: string) => void;
  onRestore: (file: string) => void;
  onFetchModels: (profile: GrokProfile) => Promise<string[] | null>;
  onTestConnection: (profile: GrokProfile) => Promise<unknown>;
  onPreviewApply: (profile: GrokProfile) => Promise<string | null>;
}) {
  const [editing, setEditing] = useState<GrokProfile | null>(null);
  const [query, setQuery] = useState("");
  const [draggedKey, setDraggedKey] = useState<string | null>(null);
  const [previewText, setPreviewText] = useState<string | null>(null);
  const [fetchedModels, setFetchedModels] = useState<string[]>([]);

  const cards = useMemo(
    () => buildProviderCards(profiles, status, layout),
    [profiles, status, layout],
  );

  const visible = useMemo(() => filterProviderCards(cards, query), [cards, query]);

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

  async function togglePin(key: string) {
    const pinned = new Set(layout.pinnedIds);
    if (pinned.has(key)) pinned.delete(key);
    else pinned.add(key);
    await onSaveLayout({
      order: cards.map((c) => c.key),
      pinnedIds: [...pinned],
    });
  }

  async function reorder(sourceKey: string, targetKey: string) {
    if (!sourceKey || sourceKey === targetKey) return;
    const source = cards.find((c) => c.key === sourceKey);
    const target = cards.find((c) => c.key === targetKey);
    if (!source || !target || source.pinned !== target.pinned) return;
    const order = cards.map((c) => c.key);
    const from = order.indexOf(sourceKey);
    const to = order.indexOf(targetKey);
    if (from < 0 || to < 0) return;
    order.splice(from, 1);
    order.splice(to, 0, sourceKey);
    await onSaveLayout({ order, pinnedIds: layout.pinnedIds });
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

      {health && health.issues.length > 0 && (
        <div className="providers-health" aria-label="Grok 配置诊断">
          <strong>
            健康诊断 · {health.activeMode}
            {health.activeProfileId ? ` · ${health.activeProfileId}` : ""}
          </strong>
          <ul>
            {health.issues.map((issue) => (
              <li key={issue.code} data-severity={issue.severity}>
                [{issue.severity}] {issue.message}
              </li>
            ))}
          </ul>
        </div>
      )}

      <div className="providers-toolbar">
        <div>
          <h2 className="providers-title">Grok 登录方式</h2>
          <p className="providers-desc">
            在官方账号与 API 供应商之间切换 `~/.grok/config.toml`；新开 Grok 会话生效
          </p>
        </div>
        <div className="providers-actions">
          <button type="button" className="btn" onClick={() => onRefresh()} disabled={loading}>
            刷新
          </button>
          <button
            type="button"
            className="btn"
            onClick={() => onApplyPrivacy()}
            disabled={!!busyId}
          >
            隐私保护
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
        placeholder="搜索官方账号、供应商名称、URL 或模型…"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />

      {status && (
        <p className="providers-meta muted">
          配置：{status.configPath}
          {status.configExists ? "" : "（文件不存在）"} · 档案目录：{status.dataDir}
          {status.officialActive ? " · 当前：官方账号" : ""}
        </p>
      )}

      {loading && !status && (
        <p className="providers-meta muted" role="status">
          加载中…
        </p>
      )}

      {visible.length === 0 ? (
        <div className="empty-state">
          <h3>没有匹配的登录方式</h3>
          <p>调整搜索词，或添加一个 API 供应商。</p>
        </div>
      ) : (
        <div className="provider-grid">
          {visible.map((card) => (
            <ProviderCardView
              key={card.key}
              card={card}
              busyId={busyId}
              draggedKey={draggedKey}
              onDragStart={setDraggedKey}
              onDragEnd={() => setDraggedKey(null)}
              onDropOn={(target) => void reorder(draggedKey ?? "", target)}
              onTogglePin={() => void togglePin(card.key)}
              onActivateOfficial={onActivateOfficial}
              onActivate={onActivate}
              onEdit={(p) => setEditing({ ...p })}
              onDelete={onDelete}
            />
          ))}
        </div>
      )}

      <section className="providers-backups">
        <h3>配置备份</h3>
        {backups.length === 0 ? (
          <p className="muted">暂无备份（启用供应商 / 官方 / 隐私保护时会自动生成）</p>
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
            <div className="providers-form-actions providers-form-tools">
              <button
                type="button"
                className="btn"
                disabled={!!busyId || !editing.baseUrl.trim()}
                onClick={() =>
                  void (async () => {
                    const models = await onFetchModels(editing);
                    if (!models) return;
                    setFetchedModels(models);
                    if (models.length > 0 && !editing.defaultModel) {
                      setEditing({
                        ...editing,
                        defaultModel: models[0],
                        webSearchModel: editing.webSearchModel || models[0],
                        subagentsDefaultModel: editing.subagentsDefaultModel || models[0],
                        availableModels: models,
                      });
                    } else {
                      setEditing({ ...editing, availableModels: models });
                    }
                  })()
                }
              >
                拉取模型
              </button>
              <button
                type="button"
                className="btn"
                disabled={!!busyId || !editing.baseUrl.trim()}
                onClick={() => void onTestConnection(editing)}
              >
                连通测试
              </button>
              <button
                type="button"
                className="btn"
                disabled={!!busyId}
                onClick={() =>
                  void (async () => {
                    const text = await onPreviewApply(editing);
                    if (text != null) setPreviewText(text);
                  })()
                }
              >
                预览 config
              </button>
            </div>
            {fetchedModels.length > 0 && (
              <label>
                已拉取模型（点击填入默认）
                <select
                  value={editing.defaultModel}
                  onChange={(e) =>
                    setEditing({
                      ...editing,
                      defaultModel: e.target.value,
                      webSearchModel: editing.webSearchModel || e.target.value,
                      subagentsDefaultModel: editing.subagentsDefaultModel || e.target.value,
                    })
                  }
                >
                  <option value="">选择模型…</option>
                  {fetchedModels.map((m) => (
                    <option key={m} value={m}>
                      {m}
                    </option>
                  ))}
                </select>
              </label>
            )}
            {previewText != null && (
              <label className="providers-preview">
                config.toml 预览（未写入）
                <textarea readOnly rows={10} value={previewText} spellCheck={false} />
              </label>
            )}
            <div className="providers-form-actions">
              <button
                type="button"
                className="btn"
                onClick={() => {
                  setEditing(null);
                  setPreviewText(null);
                  setFetchedModels([]);
                }}
              >
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

function ProviderCardView({
  card,
  busyId,
  draggedKey,
  onDragStart,
  onDragEnd,
  onDropOn,
  onTogglePin,
  onActivateOfficial,
  onActivate,
  onEdit,
  onDelete,
}: {
  card: ProviderCard;
  busyId: string | null;
  draggedKey: string | null;
  onDragStart: (key: string) => void;
  onDragEnd: () => void;
  onDropOn: (key: string) => void;
  onTogglePin: () => void;
  onActivateOfficial: () => void;
  onActivate: (id: string) => void;
  onEdit: (profile: GrokProfile) => void;
  onDelete: (id: string) => void;
}) {
  const official = card.kind === "official";
  const profile = card.profile;
  const busy =
    busyId === card.key ||
    (official && busyId === OFFICIAL_PROVIDER_KEY) ||
    (!!profile && busyId === profile.id);

  return (
    <article
      className={`provider-card${card.pinned ? " provider-card-pinned" : ""}${
        draggedKey === card.key ? " provider-card-dragging" : ""
      }`}
      data-active={card.isActive}
      data-provider-key={card.key}
      data-pinned={card.pinned ? "1" : "0"}
      onDragOver={(e) => {
        if (!draggedKey || draggedKey === card.key) return;
        const sourceEl = document.querySelector(
          `[data-provider-key="${CSS.escape(draggedKey)}"]`,
        ) as HTMLElement | null;
        if (!sourceEl || sourceEl.dataset.pinned !== (card.pinned ? "1" : "0")) return;
        e.preventDefault();
      }}
      onDrop={(e) => {
        e.preventDefault();
        onDropOn(card.key);
      }}
    >
      <header className="provider-card-head">
        <button
          type="button"
          className="provider-drag-handle"
          draggable
          title="拖动排序"
          aria-label={`拖动 ${card.name} 排序`}
          onDragStart={(e) => {
            onDragStart(card.key);
            e.dataTransfer.effectAllowed = "move";
            e.dataTransfer.setData("text/plain", card.key);
          }}
          onDragEnd={onDragEnd}
        >
          ↕
        </button>
        <div className="provider-card-info">
          <h3>{card.name}</h3>
          <p className="muted mono">{card.subtitle}</p>
        </div>
        <div className="provider-card-flags">
          {card.pinned && <span className="provider-pin-badge">已置顶</span>}
          {card.isActive && <span className="provider-badge">启用中</span>}
        </div>
      </header>
      <dl className="provider-fields">
        {official ? (
          <>
            <div>
              <dt>登录状态</dt>
              <dd>{card.loggedIn ? "已登录" : "尚未登录"}</dd>
            </div>
            <div>
              <dt>模式</dt>
              <dd>OAuth / auth.json</dd>
            </div>
          </>
        ) : (
          <>
            <div>
              <dt>默认模型</dt>
              <dd>{profile?.defaultModel || "—"}</dd>
            </div>
            <div>
              <dt>API Key</dt>
              <dd className="mono">{maskKey(profile?.apiKey ?? "")}</dd>
            </div>
          </>
        )}
      </dl>
      <footer className="provider-card-actions">
        <button
          type="button"
          className="btn sm primary"
          disabled={card.isActive || busy}
          onClick={() => {
            if (official) onActivateOfficial();
            else if (profile) onActivate(profile.id);
          }}
        >
          {card.isActive ? "已启用" : "启用"}
        </button>
        <button type="button" className="btn sm ghost" onClick={onTogglePin}>
          {card.pinned ? "取消置顶" : "置顶"}
        </button>
        {!official && profile && (
          <>
            <button type="button" className="btn sm" onClick={() => onEdit(profile)}>
              编辑
            </button>
            <button
              type="button"
              className="btn sm danger"
              disabled={busy}
              onClick={() => {
                if (confirm(`删除供应商「${profile.name}」？不会删除 config.toml`)) {
                  onDelete(profile.id);
                }
              }}
            >
              删除
            </button>
          </>
        )}
      </footer>
    </article>
  );
}
