// AgenticBoot 工具管理相关 TypeScript 类型

export interface DetectResult {
  installed: boolean;
  version?: string;
  installPath?: string;
}

export interface ToolMeta {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: 'ai-cli' | 'ai-ide' | 'local-model' | 'dependency';
}

export interface ToolDependency {
  toolId: string;
  minVersion?: string;
}

export interface ToolUpdateSource {
  kind: "npm" | "github" | string;
  id: string;
}

export interface ToolPlatformSupport {
  windows: "implemented" | "planned" | "unsupported" | string;
  macos: "implemented" | "planned" | "unsupported" | string;
  linux: "implemented" | "planned" | "unsupported" | string;
}

export interface ToolCapabilities {
  canInstall: boolean;
  canUninstall: boolean;
  canLaunch: boolean;
  canUpdate: boolean;
  supportsPathlessUninstall: boolean;
  commandName?: string;
  managedShimName?: string;
  managedExecutableCandidates: string[];
}

export interface ToolCatalogItem extends ToolMeta {
  installStrategy: string;
  dependencies: ToolDependency[];
  updateSource?: ToolUpdateSource;
  platformSupport: ToolPlatformSupport;
  capabilities: ToolCapabilities;
}

export interface InstallStep {
  toolId: string;
  toolName: string;
  category: string;
  reason: string; // "selected" | "dependency_of(Claude Code)"
  isInstalled: boolean;
}

export interface InstallPlan {
  steps: InstallStep[];
}

export interface ResolveProgress {
  toolId: string;
  toolName: string;
  phase: 'resolving' | 'resolved' | 'error';
  message: string;
}

export interface InstallProgress {
  toolId: string;
  toolName: string;
  phase: 'starting' | 'downloading' | 'extracting' | 'installing' | 'configuring' | 'complete' | 'error' | 'skipped';
  percent: number; // 0-100
  message: string;
}

export type InstallLogLevel =
  | "info"
  | "stdout"
  | "stderr"
  | "success"
  | "error";

export type InstallLogKind =
  | "session-started"
  | "phase"
  | "command"
  | "output"
  | "result";

export interface InstallLogEvent {
  toolId: string;
  toolName: string;
  sessionId: string;
  timestamp: string;
  phase?: string;
  level: InstallLogLevel;
  kind: InstallLogKind;
  line: string;
  command?: string;
  exitCode?: number | null;
  source?: "native" | "optimistic";
}

export interface InstallActivityItem {
  timestamp: string;
  kind: Extract<InstallLogKind, "phase" | "command" | "output" | "result">;
  line: string;
}

export interface ToolInstallSession {
  toolId: string;
  toolName: string;
  sessionId: string;
  status: "running" | "complete" | "error";
  source?: "native" | "optimistic";
  startedAt: string;
  endedAt?: string;
  lastSummary?: string;
  installPath?: string;
  entries: InstallLogEvent[];
  activity: InstallActivityItem[];
}

export interface InstalledTool {
  id: string;
  name: string;
  version?: string;
  installPath: string;
  installRoot: string;
  category: 'tool' | 'ai-cli' | 'ai-ide' | 'local-model' | 'dependency';
  status: 'not_installed' | 'installing' | 'installed' | 'detected' | 'error';
  stateSource?: 'managed' | 'external_detected';
  installedAt?: number;
  lastSeenAt?: number;
  updatedAt?: number;
}

export interface ToolUpdateInfo {
  toolId: string;
  currentVersion: string;
  latestVersion: string;
}
