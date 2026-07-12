export type CliType = "codex" | "claude-code" | "cursor" | "grok-build";
export type TerminalType = "system" | "iterm2" | "ghostty";
export type LaunchMode = "new-tab" | "new-window";
export type ThemeMode = "dark" | "light" | "system";
export type StatusType = "info" | "success" | "error";
export type AppTool = "sessions" | "ports";
export type PortProtocol = "tcp" | "udp";
export type PortScope = "project" | "all";

export interface SessionData {
  id: string;
  cliType: CliType;
  sessionId: string;
  projectDir: string;
  projectName: string;
  lastActiveAt: string;
  summary?: string | null;
}

export interface CliScanError {
  cliType: CliType;
  message: string;
}

export interface ScanResponse {
  sessions: SessionData[];
  scanErrors: CliScanError[];
}

export interface PortUsage {
  id: string;
  command: string;
  pid: number;
  user: string;
  protocol: PortProtocol;
  address: string;
  port: number;
  state: string;
  executablePath: string;
  workingDirectory: string;
  parentCommand: string;
  isProjectService: boolean;
  userOwned: boolean;
}

export interface PortScanResponse {
  ports: PortUsage[];
  rawLineCount: number;
  commandDescription: string;
  scannedAt: string;
}

export const CLI_ORDER: CliType[] = ["codex", "claude-code", "cursor", "grok-build"];

export const CLI_LABELS: Record<CliType, string> = {
  codex: "Codex",
  "claude-code": "Claude Code",
  cursor: "Cursor",
  "grok-build": "Grok Build",
};

export const TERMINAL_LABELS: Record<TerminalType, string> = {
  system: "Terminal.app",
  iterm2: "iTerm2",
  ghostty: "Ghostty",
};

export const LAUNCH_MODE_LABELS: Record<LaunchMode, string> = {
  "new-tab": "新标签页",
  "new-window": "新窗口",
};

export const THEME_MODE_OPTIONS: ThemeMode[] = ["dark", "light", "system"];

export const THEME_MODE_LABELS: Record<ThemeMode, string> = {
  dark: "黑",
  light: "白",
  system: "跟随系统",
};

export const APP_TOOL_LABELS: Record<AppTool, string> = {
  sessions: "Session",
  ports: "Port",
};

export const PORT_SCOPE_LABELS: Record<PortScope, string> = {
  project: "项目服务",
  all: "全部端口",
};

export const PORT_PROTOCOL_LABELS: Record<PortProtocol, string> = {
  tcp: "TCP",
  udp: "UDP",
};
