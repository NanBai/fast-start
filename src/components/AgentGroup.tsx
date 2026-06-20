import { useState } from "react";
import { BrandMark } from "./icons/BrandMark";
import { Icon } from "./icons/Icon";
import { ProjectBucket } from "./ProjectBucket";
import { CLI_LABELS, CliType, SessionData } from "../types";

const AGENT_HINTS: Record<CliType, string> = {
  codex: "Codex CLI 历史会话",
  "claude-code": "Claude Code 项目会话",
  cursor: "Cursor Agent 工作区会话",
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
  launchingId,
  onLaunch,
}: {
  cliType: CliType;
  sessions: SessionData[];
  launchingId: string | null;
  onLaunch: (sessionId: string) => Promise<void>;
}) {
  const [expanded, setExpanded] = useState(false);
  const projectGroups = groupByProjectDir(sessions);

  return (
    <section className="cli-group" data-cli={cliType} data-open={expanded}>
      <button
        type="button"
        className="cli-group-header"
        onClick={() => setExpanded((current) => !current)}
        aria-expanded={expanded}
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
      {expanded && (
        <div className="cli-group-card">
          <div className="cli-group-body">
            {projectGroups.length === 0 ? (
              <p className="state-line">暂无 session</p>
            ) : (
              projectGroups.map((group) => (
                <ProjectBucket
                  key={group.projectDir}
                  projectDir={group.projectDir}
                  projectName={group.projectName}
                  sessions={group.sessions}
                  launchingId={launchingId}
                  onLaunch={onLaunch}
                />
              ))
            )}
          </div>
        </div>
      )}
    </section>
  );
}
