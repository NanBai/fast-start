export type CliType = "codex" | "claude-code" | "cursor";
export type TerminalType = "system" | "iterm2" | "ghostty";

export interface SessionData {
  id: string;
  cliType: CliType;
  sessionId: string;
  projectDir: string;
  projectName: string;
  lastActiveAt: string;
}

export interface CliScanError {
  cliType: CliType;
  message: string;
}

export interface ScanResponse {
  sessions: SessionData[];
  scanErrors: CliScanError[];
}

export const CLI_ORDER: CliType[] = ["codex", "claude-code", "cursor"];

export const CLI_LABELS: Record<CliType, string> = {
  codex: "Codex",
  "claude-code": "Claude Code",
  cursor: "Cursor",
};

export const TERMINAL_LABELS: Record<TerminalType, string> = {
  system: "Terminal.app",
  iterm2: "iTerm2",
  ghostty: "Ghostty",
};
