import { ThemeMenu } from "./Controls";
import { Icon } from "./icons/Icon";
import { ProvidersWorkspace } from "./ProvidersWorkspace";
import { Skeleton } from "./Skeleton";
import type {
  GrokBackupInfo,
  GrokHealthReport,
  GrokProfile,
  GrokProviderLayout,
  GrokProviderStatus,
  ThemeMode,
} from "../types";

export type ProvidersToolPanelProps = {
  profiles: GrokProfile[];
  status: GrokProviderStatus | null;
  backups: GrokBackupInfo[];
  health: GrokHealthReport | null;
  layout: GrokProviderLayout;
  loading: boolean;
  busyId: string | null;
  themeMode: ThemeMode;
  onThemeModeChange: (mode: ThemeMode) => void | Promise<void>;
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
};

/** Providers 工具页：控制栏 + ProvidersWorkspace（Grok 区）。 */
export function ProvidersToolPanel({
  profiles,
  status,
  backups,
  health,
  layout,
  loading,
  busyId,
  themeMode,
  onThemeModeChange,
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
}: ProvidersToolPanelProps) {
  return (
    <>
      <div className="control-bar">
        <div className="control-bar-main">
          <div className="control-groups control-groups-providers">
            <section className="control-group control-group-hint" aria-label="说明">
              <div className="control-group-label">说明</div>
              <div className="control-group-body">
                <p className="providers-control-hint muted">
                  切换后<strong>新开</strong>对应 CLI 会话才会读取新配置；不会结束已运行的会话。
                </p>
              </div>
            </section>
            <section className="control-group" aria-label="外观">
              <div className="control-group-label">
                <Icon.Appearance />
                外观
              </div>
              <div className="control-group-body">
                <ThemeMenu
                  value={themeMode}
                  onChange={async (mode) => {
                    await onThemeModeChange(mode);
                  }}
                />
              </div>
            </section>
          </div>
        </div>
      </div>

      <div className="workspace-panel">
        {loading && status == null ? (
          <Skeleton />
        ) : (
          <ProvidersWorkspace
            profiles={profiles}
            status={status}
            backups={backups}
            health={health}
            layout={layout}
            loading={loading}
            busyId={busyId}
            onRefresh={onRefresh}
            onActivate={onActivate}
            onActivateOfficial={onActivateOfficial}
            onApplyPrivacy={onApplyPrivacy}
            onSaveLayout={onSaveLayout}
            onImport={onImport}
            onSave={onSave}
            onDelete={onDelete}
            onRestore={onRestore}
            onFetchModels={onFetchModels}
            onTestConnection={onTestConnection}
            onPreviewApply={onPreviewApply}
          />
        )}
      </div>
    </>
  );
}
