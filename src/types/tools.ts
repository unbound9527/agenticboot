// AgenticBoot 工具管理相关 TypeScript 类型

export interface NetworkStatus {
  githubReachable: boolean;
  npmReachable: boolean;
  youtubeReachable: boolean;
  errorMessage?: string;
}

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

export interface InstallProgress {
  toolId: string;
  toolName: string;
  phase: 'starting' | 'downloading' | 'extracting' | 'installing' | 'configuring' | 'complete' | 'error' | 'skipped';
  percent: number; // 0-100
  message: string;
}

export interface InstalledTool {
  id: string;
  name: string;
  version?: string;
  installPath: string;
  installRoot: string;
  category: 'tool' | 'dependency';
  status: 'not_installed' | 'installing' | 'installed' | 'error';
  installedAt?: number;
  updatedAt?: number;
}

export interface ToolUpdateInfo {
  toolId: string;
  currentVersion: string;
  latestVersion: string;
}
