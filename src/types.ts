export type CliType = "codex" | "claude-code" | "cursor" | "grok-build" | "opencode";
export type TerminalType = "system" | "iterm2" | "ghostty";
export type LaunchMode = "new-tab" | "new-window";
export type SessionListMode = "by-agent" | "by-project";
export type ThemeMode = "dark" | "light" | "system";
export type StatusType = "info" | "success" | "error";
export type AppTool = "sessions" | "ports" | "providers";
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
  /** 本次结果是否来自磁盘 scan-cache（冷启动秒开） */
  fromCache?: boolean;
  /** 完整扫描耗时（毫秒）；缓存命中时为上次 full scan 记录 */
  scanDurationMs?: number;
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

export const CLI_ORDER: CliType[] = [
  "codex",
  "claude-code",
  "cursor",
  "grok-build",
  "opencode",
];

export const CLI_LABELS: Record<CliType, string> = {
  codex: "Codex",
  "claude-code": "Claude Code",
  cursor: "Cursor",
  "grok-build": "Grok Build",
  opencode: "OpenCode",
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

export const SESSION_LIST_MODE_LABELS: Record<SessionListMode, string> = {
  "by-agent": "按 Agent",
  "by-project": "按项目",
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
  providers: "Grok",
};

export interface GrokModelDef {
  name: string;
  model: string;
  baseUrl: string;
  apiKey: string;
  apiBackend: string;
  extraHeaders: Record<string, string>;
  supportsBackendSearch: boolean;
  contextWindow: number;
  maxCompletionTokens: number;
}

export interface GrokProfile {
  id: string;
  name: string;
  upstreamFormat: string;
  baseUrl: string;
  apiKey: string;
  availableModels: string[];
  defaultModel: string;
  webSearchModel: string;
  subagentsDefaultModel: string;
  models: GrokModelDef[];
  createdAt?: string | null;
  updatedAt?: string | null;
  isActive: boolean;
}

export interface GrokBackupInfo {
  file: string;
  path: string;
  createdAt: string;
  size: number;
}

export interface GrokProviderStatus {
  activeProfile: GrokProfile | null;
  configPath: string;
  dataDir: string;
  configMatchesActive: boolean;
  configExists: boolean;
  officialActive: boolean;
  officialLoggedIn: boolean;
}

export interface GrokActivateOfficialResult {
  loginRequired: boolean;
  message: string;
}

export interface GrokPrivacyResult {
  path: string;
  message: string;
}

export interface GrokFetchModelsResult {
  models: string[];
}

export interface GrokTestConnectionResult {
  ok: boolean;
  latencyMs: number;
  message: string;
}

export interface GrokProviderLayout {
  order: string[];
  pinnedIds: string[];
}

export function emptyGrokProfile(): GrokProfile {
  return {
    id: "",
    name: "",
    upstreamFormat: "openai_chat",
    baseUrl: "",
    apiKey: "",
    availableModels: [],
    defaultModel: "",
    webSearchModel: "",
    subagentsDefaultModel: "",
    models: [],
    isActive: false,
  };
}

export const PORT_SCOPE_LABELS: Record<PortScope, string> = {
  project: "项目服务",
  all: "全部端口",
};

export const PORT_PROTOCOL_LABELS: Record<PortProtocol, string> = {
  tcp: "TCP",
  udp: "UDP",
};
