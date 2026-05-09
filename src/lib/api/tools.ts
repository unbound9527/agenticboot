// AgenticBoot 工具管理 Tauri 命令调用和事件监听

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  NetworkStatus,
  DetectResult,
  InstallPlan,
  InstallProgress,
  InstalledTool,
  ToolUpdateInfo,
} from "@/types/tools";

const DETECT_TOOLS_TIMEOUT_MS = 12000;

export function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  message: string,
): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = window.setTimeout(() => {
      reject(new Error(message));
    }, timeoutMs);

    promise.then(
      (value) => {
        window.clearTimeout(timer);
        resolve(value);
      },
      (error) => {
        window.clearTimeout(timer);
        reject(error);
      },
    );
  });
}

export const toolsApi = {
  // ── 命令调用 ──

  checkNetwork(): Promise<NetworkStatus> {
    return invoke("check_network");
  },

  detectTools(
    toolIds: string[],
    installRoot?: string,
    forceRefresh = false,
  ): Promise<DetectResult[]> {
    return withTimeout(
      invoke("detect_tools", {
        toolIds,
        installRoot: installRoot ?? null,
        forceRefresh,
      }),
      DETECT_TOOLS_TIMEOUT_MS,
      "Tool detection timed out",
    );
  },

  resolveInstallPlan(
    toolIds: string[],
    installRoot?: string,
  ): Promise<InstallPlan> {
    return invoke("resolve_install_plan", {
      toolIds,
      installRoot: installRoot ?? null,
    });
  },

  executeInstallPlan(plan: InstallPlan, rootPath: string): Promise<void> {
    return invoke("execute_install_plan_with_plan", { plan, rootPath });
  },

  uninstallTool(toolId: string, rootPath: string): Promise<void> {
    return invoke("uninstall_tool", { toolId, rootPath });
  },

  getInstalledTools(): Promise<InstalledTool[]> {
    return invoke("get_installed_tools");
  },

  hasAnyInstalledTools(): Promise<boolean> {
    return invoke("has_any_installed_tools");
  },

  getInstallRoot(): Promise<string | null> {
    return invoke("get_install_root");
  },

  setInstallRoot(path: string): Promise<void> {
    return invoke("set_install_root", { path });
  },

  checkToolUpdates(): Promise<ToolUpdateInfo[]> {
    return invoke("check_tool_updates");
  },

  // ── 事件监听 ──

  onInstallProgress(
    callback: (progress: InstallProgress) => void,
  ): Promise<UnlistenFn> {
    return listen<InstallProgress>("install-progress", (event) => {
      callback(event.payload);
    });
  },

  onInstallComplete(callback: (toolId: string) => void): Promise<UnlistenFn> {
    return listen<string>("install-complete", (event) => {
      callback(event.payload);
    });
  },

  onInstallError(
    callback: (error: { toolId: string; error: string }) => void,
  ): Promise<UnlistenFn> {
    return listen<{ toolId: string; error: string }>(
      "install-error",
      (event) => {
        callback(event.payload);
      },
    );
  },
};
