// AgenticBoot 工具管理 Tauri 命令调用和事件监听

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  NetworkStatus,
  DetectResult,
  InstallPlan,
  InstallProgress,
  InstalledTool,
  ToolUpdateInfo,
} from '@/types/tools';

export const toolsApi = {
  // ── 命令调用 ──

  checkNetwork(): Promise<NetworkStatus> {
    return invoke('check_network');
  },

  detectTools(
    toolIds: string[],
    installRoot?: string,
    forceRefresh = false
  ): Promise<DetectResult[]> {
    return invoke('detect_tools', {
      toolIds,
      installRoot: installRoot ?? null,
      forceRefresh,
    });
  },

  resolveInstallPlan(toolIds: string[], installRoot?: string): Promise<InstallPlan> {
    return invoke('resolve_install_plan', { toolIds, installRoot: installRoot ?? null });
  },

  executeInstallPlan(plan: InstallPlan, rootPath: string): Promise<void> {
    return invoke('execute_install_plan_with_plan', { plan, rootPath });
  },

  uninstallTool(toolId: string, rootPath: string): Promise<void> {
    return invoke('uninstall_tool', { toolId, rootPath });
  },

  getInstalledTools(): Promise<InstalledTool[]> {
    return invoke('get_installed_tools');
  },

  hasAnyInstalledTools(): Promise<boolean> {
    return invoke('has_any_installed_tools');
  },

  getInstallRoot(): Promise<string | null> {
    return invoke('get_install_root');
  },

  setInstallRoot(path: string): Promise<void> {
    return invoke('set_install_root', { path });
  },

  checkToolUpdates(): Promise<ToolUpdateInfo[]> {
    return invoke('check_tool_updates');
  },

  // ── 事件监听 ──

  onInstallProgress(
    callback: (progress: InstallProgress) => void
  ): Promise<UnlistenFn> {
    return listen<InstallProgress>('install-progress', (event) => {
      callback(event.payload);
    });
  },

  onInstallComplete(
    callback: (toolId: string) => void
  ): Promise<UnlistenFn> {
    return listen<string>('install-complete', (event) => {
      callback(event.payload);
    });
  },

  onInstallError(
    callback: (error: { toolId: string; error: string }) => void
  ): Promise<UnlistenFn> {
    return listen<{ toolId: string; error: string }>(
      'install-error',
      (event) => {
        callback(event.payload);
      }
    );
  },
};
