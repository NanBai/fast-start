import type { MouseEvent } from "react";
import { useState } from "react";
import { BrandMark } from "./icons/BrandMark";
import { Icon } from "./icons/Icon";
import { ProjectBucket } from "./ProjectBucket";
import { CLI_LABELS, CliType, SessionData } from "../types";

const AGENT_HINTS: Record<CliType, string> = {
  codex: "Codex CLI 历史会话",
  "claude-code": "Claude Code 项目会话",
  cursor: "Cursor Agent 工作区会话",
  "grok-build": "Grok Build 历史会话",
  opencode: "OpenCode 历史会话",
};

type ProjectSessionGroup = {
  projectDir: string;
  projectName: string;
  sessions: SessionData[];
};

function groupByProjectDir(sessions: SessionData[]): ProjectSessionGroup[] {
  const groups: ProjectSessionGroup[] = [];
  const groupIndex = new Map<string, number>();
  for (const session of sessions) {
    const existing = groupIndex.get(session.projectDir);
    if (existing === undefined) {
      groupIndex.set(session.projectDir, groups.length);
      groups.push({
        projectDir: session.projectDir,
        projectName: session.projectName,
        sessions: [session],
      });
    } else {
      groups[existing].sessions.push(session);
    }
  }
  return groups;
}

// 先按 agent 分区，再在每个 agent 下按工作目录聚合历史会话。
export function AgentGroup({
  cliType,
  sessions,
  favoriteProjectDirs,
  favoriteSessionIds,
  forceOpen = false,
  scanError = null,
  activeSessionId,
  launchingId,
  deletingId,
  onLaunch,
  onToggleFavoriteProject,
  onToggleSessionFavorite,
  onSessionContextMenu,
}: {
  cliType: CliType;
  sessions: SessionData[];
  favoriteProjectDirs: Set<string>;
  favoriteSessionIds: Set<string>;
  forceOpen?: boolean;
  scanError?: string | null;
  activeSessionId: string | null;
  launchingId: string | null;
  deletingId: string | null;
  onLaunch: (sessionId: string) => Promise<void>;
  onToggleFavoriteProject: (projectDir: string) => void;
  onToggleSessionFavorite: (sessionId: string) => void;
  onSessionContextMenu: (
    session: SessionData,
    event: MouseEvent<HTMLDivElement>,
  ) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const projectGroups = groupByProjectDir(sessions);
  const open = forceOpen || expanded;

  return (
    <section className="cli-group" data-cli={cliType} data-open={open}>
      <button
        type="button"
        className="cli-group-header"
        onClick={() => setExpanded((current) => !current)}
        aria-expanded={open}
      >
        <span className="cli-mark" data-cli={cliType} aria-hidden="true">
          <BrandMark cliType={cliType} />
        </span>
        <span className="agent-title">
          <span className="agent-eyebrow">AGENT</span>
          <span className="agent-name">{CLI_LABELS[cliType]}</span>
          <span className="agent-summary">{AGENT_HINTS[cliType]}</span>
        </span>
        <span className="agent-stats">
          <span className="agent-stat">
            <strong>{projectGroups.length}</strong>
            <span>目录</span>
          </span>
          <span className="agent-stat">
            <strong>{sessions.length}</strong>
            <span>会话</span>
          </span>
        </span>
        <span className="chev-group">
          <Icon.Chevron />
        </span>
      </button>
      {open && (
        <div className="cli-group-card">
          <div className="cli-group-body">
            {projectGroups.length === 0 ? (
              <p className="state-line">
                {scanError
                  ? `扫描失败：${scanError}。可点右上角刷新重试。`
                  : `暂无 ${CLI_LABELS[cliType]} session。在本机使用该 CLI 后刷新即可出现。`}
              </p>
            ) : (
              projectGroups.map((group) => (
                <ProjectBucket
                  key={group.projectDir}
                  projectDir={group.projectDir}
                  projectName={group.projectName}
                  sessions={group.sessions}
                  favorite={favoriteProjectDirs.has(group.projectDir)}
                  forceOpen={forceOpen}
                  favoriteSessionIds={favoriteSessionIds}
                  activeSessionId={activeSessionId}
                  launchingId={launchingId}
                  deletingId={deletingId}
                  onLaunch={onLaunch}
                  onToggleFavorite={onToggleFavoriteProject}
                  onToggleSessionFavorite={onToggleSessionFavorite}
                  onSessionContextMenu={onSessionContextMenu}
                />
              ))
            )}
          </div>
        </div>
      )}
    </section>
  );
}
