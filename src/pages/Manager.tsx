import { useState, useCallback, useEffect, useMemo } from "react";
import { FolderOpen, RefreshCw, Settings } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { useQueryClient } from "@tanstack/react-query";
import { InstallConsole } from "@/components/tools/InstallConsole";
import { ToolCard } from "@/components/tools/ToolCard";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useInstallProgress } from "@/hooks/useInstallProgress";
import { useInstallSessions } from "@/hooks/useInstallSessions";
import {
  useExecuteInstallPlan,
  useInstalledTools,
  useInstallRoot,
  useResolveInstallPlan,
  useToolCatalog,
  useToolUpdates,
  useUninstallTool,
} from "@/hooks/useTools";
import { toolsApi } from "@/lib/api/tools";
import type { DetectResult, InstalledTool, ToolCatalogItem } from "@/types/tools";

interface ManagerProps {
  onInstallMore?: () => void;
  onToolStateChanged?: () => void;
}

function toExternalInstalledTool(
  meta: ToolCatalogItem,
  detect: DetectResult,
  fallbackRoot: string,
): InstalledTool {
  return {
    id: meta.id,
    name: meta.name,
    version: detect.version,
    installPath: detect.installPath ?? "",
    installRoot: detect.installPath ?? fallbackRoot,
    category: meta.category,
    status: "detected",
  };
}

function canAutoUninstallTool(
  tool: InstalledTool,
  meta: ToolCatalogItem | undefined,
) {
  if (tool.category === "dependency") {
    return false;
  }

  if (
    !tool.installPath?.trim() &&
    !(tool.status === "detected" && meta?.capabilities.supportsPathlessUninstall)
  ) {
    return false;
  }

  if (tool.status === "installed") {
    return true;
  }

  if (tool.status === "detected") {
    return true;
  }

  return false;
}

function buildPendingInstallProgress(tool: ToolCatalogItem) {
  return {
    toolId: tool.id,
    toolName: tool.name,
    phase: "starting" as const,
    percent: 0,
    message: "Preparing installation...",
  };
}

function isManagedUserTool(tool: ToolCatalogItem) {
  return (
    tool.category !== "dependency" &&
    (tool.capabilities.canInstall ||
      tool.capabilities.canUninstall ||
      tool.capabilities.canLaunch)
  );
}

function addToolId(previous: Set<string>, toolId: string) {
  const next = new Set(previous);
  next.add(toolId);
  return next;
}

function removeToolId(previous: Set<string>, toolId: string) {
  const next = new Set(previous);
  next.delete(toolId);
  return next;
}

export function Manager({ onInstallMore, onToolStateChanged }: ManagerProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [activeTab, setActiveTab] = useState("installed");
  const [editRoot, setEditRoot] = useState("");
  const [activeInstallRoot, setActiveInstallRoot] = useState("");
  const [openConsoleToolId, setOpenConsoleToolId] = useState<
    string | null
  >(null);
  const [detectedTools, setDetectedTools] = useState<
    Record<string, DetectResult>
  >({});
  const [isDetecting, setIsDetecting] = useState(false);
  const [isCheckingUpdates, setIsCheckingUpdates] = useState(false);
  const [uninstallingToolIds, setUninstallingToolIds] = useState<Set<string>>(
    () => new Set(),
  );
  const [launchingToolId, setLaunchingToolId] = useState<string | null>(null);
  const [pendingActionToolIds, setPendingActionToolIds] = useState<Set<string>>(
    () => new Set(),
  );

  const {
    data: toolCatalog = [],
    isError: isToolCatalogError,
    refetch: refetchToolCatalog,
  } = useToolCatalog();
  const visibleTools = useMemo(
    () => toolCatalog.filter(isManagedUserTool),
    [toolCatalog],
  );
  const { data: installedTools = [] } = useInstalledTools();
  const { data: installRoot } = useInstallRoot();
  const { data: updates = [], refetch: refetchToolUpdates } = useToolUpdates();
  const uninstallTool = useUninstallTool();
  const resolvePlan = useResolveInstallPlan();
  const executePlan = useExecuteInstallPlan();
  const { getToolProgress, getToolTargetProgress, resetProgress } =
    useInstallProgress();
  const {
    sessions: installSessions,
    startOptimisticSession,
    appendOptimisticEntry,
    markSessionError,
  } = useInstallSessions();
  const managedInstalledTools = installedTools.filter(
    (tool) => tool.status === "installed",
  );
  const allToolMetaById = useMemo(
    () => new Map(visibleTools.map((tool) => [tool.id, tool])),
    [visibleTools],
  );
  const updatesByToolId = new Map(updates.map((update) => [update.toolId, update]));
  const effectiveInstallRoot = activeInstallRoot || installRoot || "";
  const selectedConsoleSession = openConsoleToolId
    ? installSessions.get(openConsoleToolId) ?? null
    : null;
  const visibleConsoleSession =
    selectedConsoleSession?.status === "running" ? selectedConsoleSession : null;

  useEffect(() => {
    const nextRoot = installRoot ?? "";
    setEditRoot(nextRoot);
    setActiveInstallRoot(nextRoot);
  }, [installRoot]);

  const refreshDetectedTools = useCallback(
    (forceRefresh = false) => {
      const ids = visibleTools.map((tool) => tool.id);
      if (ids.length === 0) {
        setDetectedTools({});
        setIsDetecting(false);
        return Promise.resolve();
      }
      setIsDetecting(true);
      const detectPromise = toolsApi.detectTools(
        ids,
        effectiveInstallRoot || undefined,
        forceRefresh,
      );

      return detectPromise
        .then((results) => {
          const next: Record<string, DetectResult> = {};
          results.forEach((result, index) => {
            next[ids[index]] = result;
          });
          setDetectedTools(next);
        })
        .finally(() => {
          setIsDetecting(false);
        });
    },
    [effectiveInstallRoot, visibleTools],
  );

  useEffect(() => {
    let cancelled = false;
    const timer = setTimeout(() => {
      refreshDetectedTools().catch(() => {
        if (!cancelled) {
          setDetectedTools({});
        }
      });
    }, 300);

    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [refreshDetectedTools]);

  const managedInstalledIds = new Set(
    managedInstalledTools.map((tool) => tool.id),
  );
  const detectedInstalledIds = new Set(
    Object.entries(detectedTools)
      .filter(([, result]) => result.installed)
      .map(([toolId]) => toolId),
  );
  const installedIds = new Set([
    ...managedInstalledIds,
    ...detectedInstalledIds,
  ]);

  const mergedInstalledTools: InstalledTool[] = [
    ...managedInstalledTools,
    ...visibleTools.filter((meta) => !managedInstalledIds.has(meta.id))
      .filter((meta) => detectedTools[meta.id]?.installed)
      .map((meta) =>
        toExternalInstalledTool(meta, detectedTools[meta.id], effectiveInstallRoot),
      ),
  ];

  const notInstalled = visibleTools.filter(
    (meta) => !installedIds.has(meta.id),
  );

  const handleUninstall = useCallback(
    async (toolId: string, rootPath: string) => {
      const toolName = allToolMetaById.get(toolId)?.name ?? toolId;
      setUninstallingToolIds((previous) => addToolId(previous, toolId));
      try {
        await uninstallTool.mutateAsync({ toolId, rootPath });
        toast.success(t("tools.uninstalled", "卸载成功"));
        onToolStateChanged?.();
        await refreshDetectedTools(true).catch(() => {});
      } catch (err) {
        markSessionError(
          toolId,
          toolName,
          `System: Install request failed before completion: ${String(err)}`,
        );
        toast.error(
          t("tools.uninstallFailed", "卸载失败: {{error}}", {
            error: String(err),
          }),
        );
      } finally {
        setUninstallingToolIds((previous) => removeToolId(previous, toolId));
      }
    },
    [allToolMetaById, markSessionError, onToolStateChanged, refreshDetectedTools, uninstallTool, t],
  );

  const handleSingleInstall = useCallback(
    async (
      toolId: string,
      rootPath?: string,
      options?: { openConsole?: boolean },
    ) => {
      const resolvedRoot = (rootPath ?? effectiveInstallRoot).trim();
      if (!resolvedRoot) {
        toast.error(t("tools.installRootRequired", "请先设置安装根目录"));
        return;
      }

      if (options?.openConsole) {
        setOpenConsoleToolId(toolId);
      }
      const toolName = allToolMetaById.get(toolId)?.name ?? toolId;
      setPendingActionToolIds((previous) => addToolId(previous, toolId));
      startOptimisticSession(toolId, toolName, [
        "System: Install requested.",
        "System: Resolving install plan...",
      ]);
      try {
        const plan = await resolvePlan.mutateAsync({
          toolIds: [toolId],
          installRoot: resolvedRoot,
        });
        appendOptimisticEntry(
          toolId,
          toolName,
          "System: Install plan resolved. Starting installer...",
        );
        resetProgress();
        appendOptimisticEntry(
          toolId,
          toolName,
          "System: Waiting for installer process to start...",
        );
        await executePlan.mutateAsync({ plan, rootPath: resolvedRoot });
        toast.success(t("tools.installStarted", "安装成功"));
      } catch (err) {
        toast.error(
          t("tools.installFailed", "安装失败: {{error}}", {
            error: String(err),
          }),
        );
      } finally {
        setPendingActionToolIds((previous) => removeToolId(previous, toolId));
      }
    },
    [
      allToolMetaById,
      appendOptimisticEntry,
      effectiveInstallRoot,
      executePlan,
      markSessionError,
      resetProgress,
      resolvePlan,
      startOptimisticSession,
      t,
    ],
  );

  const persistInstallRoot = useCallback(
    async (nextRoot: string) => {
      const normalizedRoot = nextRoot.trim();
      if (!normalizedRoot) {
        setEditRoot(effectiveInstallRoot);
        return;
      }

      if (normalizedRoot === effectiveInstallRoot) {
        setEditRoot(normalizedRoot);
        return;
      }

      const previousRoot = effectiveInstallRoot;
      setEditRoot(normalizedRoot);
      setActiveInstallRoot(normalizedRoot);
      queryClient.setQueryData(["install-root"], normalizedRoot);

      try {
        await toolsApi.setInstallRoot(normalizedRoot);
      } catch (error) {
        setEditRoot(previousRoot);
        setActiveInstallRoot(previousRoot);
        queryClient.setQueryData(["install-root"], previousRoot);
        toast.error(
          t("tools.installRootSaveFailed", "保存安装根目录失败: {{error}}", {
            error: String(error),
          }),
        );
      }
    },
    [effectiveInstallRoot, queryClient, t],
  );

  const getDisplayProgress = useCallback(
    (toolId: string) => {
      const progress = getToolProgress(toolId);
      if (progress) {
        return progress;
      }

      if (pendingActionToolIds.has(toolId)) {
        const tool = allToolMetaById.get(toolId);
        if (tool) {
          return buildPendingInstallProgress(tool);
        }
      }

      return null;
    },
    [allToolMetaById, getToolProgress, pendingActionToolIds],
  );

  return (
    <div className="px-5 py-5">
      <div className="mb-5 flex items-center justify-between">
        <h1 className="text-[15px] font-semibold">
          {t("tools.manager", "软件管家")}
        </h1>
        <div className="flex gap-2">
          <Button
            variant="secondary"
            size="sm"
            onClick={() => {
              refreshDetectedTools(true).catch(() => {});
            }}
            className="text-[13px]"
            disabled={isDetecting}
          >
            <RefreshCw className={`mr-1.5 h-3 w-3 ${isDetecting ? "animate-spin" : ""}`} />
            {t("tools.refreshDetection", "重新检测")}
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => {
              setIsCheckingUpdates(true);
              refetchToolUpdates().then(({ data }) => {
                const nextUpdates = data ?? [];
                if (nextUpdates.length > 0) {
                  toast.info(
                    t("tools.updatesAvailable", "{{count}} 个工具可更新", {
                      count: nextUpdates.length,
                    }),
                  );
                } else {
                  toast.success(t("tools.allUpToDate", "所有工具均为最新版本"));
                }
              }).catch((error) => {
                toast.error(
                  t("tools.checkUpdatesFailed", "妫€鏌ユ洿鏂板け璐? {{error}}", {
                    error: String(error),
                  }),
                );
              }).finally(() => {
                setIsCheckingUpdates(false);
              });
            }}
            className="text-[13px]"
          >
            <RefreshCw className={`mr-1.5 h-3 w-3 ${isCheckingUpdates ? "animate-spin" : ""}`} />
            {t("tools.checkUpdates", "检查更新")}
          </Button>
        </div>
      </div>

      {isToolCatalogError && (
        <Alert variant="destructive" className="mb-4">
          <RefreshCw className="h-4 w-4" />
          <AlertTitle>
            {t("tools.toolCatalogLoadFailed", "Failed to load tool catalog.")}
          </AlertTitle>
          <AlertDescription className="space-y-3">
            <p>
              {t(
                "tools.toolCatalogManagerLoadFailedHint",
                "Tool management is unavailable until the catalog can be loaded.",
              )}
            </p>
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                void refetchToolCatalog();
              }}
            >
              {t("common.retry", "Retry")}
            </Button>
          </AlertDescription>
        </Alert>
      )}

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="mb-4 w-full">
          <TabsTrigger value="installed" className="flex-1 text-[13px]">
            {t("tools.installedTab", "已安装")} ({mergedInstalledTools.length})
          </TabsTrigger>
          <TabsTrigger value="available" className="flex-1 text-[13px]">
            {t("tools.availableTab", "未安装")} ({notInstalled.length})
          </TabsTrigger>
        </TabsList>

        <TabsContent value="installed" className="space-y-2">
          {mergedInstalledTools.length === 0 && (
            <div className="py-10 text-center text-muted-foreground">
              <p className="text-[14px]">
                {t("tools.noToolsInstalled", "暂无已安装工具")}
              </p>
              <Button
                variant="link"
                onClick={onInstallMore}
                className="mt-2 text-[13px]"
              >
                {t("tools.goInstall", "去安装")}
              </Button>
            </div>
          )}
          {mergedInstalledTools.map((tool) => {
            const meta = allToolMetaById.get(tool.id);
            const canUninstall = canAutoUninstallTool(tool, meta);
            const canLaunch =
              tool.installPath?.trim() && meta?.capabilities.canLaunch;

            return (
              <ToolCard
                key={tool.id}
                tool={tool}
                variant="installed"
                isUninstalling={uninstallingToolIds.has(tool.id)}
                isLaunching={launchingToolId === tool.id}
                isUpdating={pendingActionToolIds.has(tool.id)}
                progress={getDisplayProgress(tool.id)}
                installSession={installSessions.get(tool.id) ?? null}
                onOpenFolder={
                  tool.installPath
                    ? () => {
                        if (tool.installPath) {
                          toolsApi.openFolder(tool.installPath);
                        }
                      }
                    : undefined
                }
                onLaunch={
                  canLaunch
                    ? async () => {
                        if (!tool.installPath) return;
                        setLaunchingToolId(tool.id);
                        try {
                          await toolsApi.launchDesktopTool(tool.installPath);
                        } finally {
                          setLaunchingToolId(null);
                        }
                      }
                    : undefined
                }
                onUninstall={
                  canUninstall
                    ? () => handleUninstall(tool.id, tool.installRoot)
                    : undefined
                }
                onUpdate={
                  updatesByToolId.has(tool.id)
                    ? () =>
                        handleSingleInstall(tool.id, tool.installRoot, {
                          openConsole: true,
                        })
                    : undefined
                }
                onShowConsole={() =>
                  setOpenConsoleToolId((current) =>
                    current === tool.id ? null : tool.id,
                  )
                }
              />
            );
          })}
        </TabsContent>

        <TabsContent value="available" className="space-y-2">
          {notInstalled.map((tool) => (
            <ToolCard
              key={tool.id}
              tool={tool}
              variant="available"
              progress={getDisplayProgress(tool.id)}
              installSession={installSessions.get(tool.id) ?? null}
              onInstall={() => {
                handleSingleInstall(tool.id, effectiveInstallRoot, {
                  openConsole: true,
                });
              }}
              onShowConsole={() =>
                setOpenConsoleToolId((current) =>
                  current === tool.id ? null : tool.id,
                )
              }
            />
          ))}
        </TabsContent>
      </Tabs>

      {visibleConsoleSession && (
        <div className="mt-4">
          <InstallConsole
            session={visibleConsoleSession}
            progress={
              openConsoleToolId
                ? getToolTargetProgress(openConsoleToolId) ??
                  getDisplayProgress(openConsoleToolId)
                : null
            }
          />
        </div>
      )}

      <div className="mt-6 border-t border-border/50 pt-5">
        <div className="flex items-center gap-3 text-[13px] text-muted-foreground">
          <Settings className="h-4 w-4 flex-shrink-0" />
          <span className="flex-shrink-0 font-medium">
            {t("tools.installRoot", "安装根目录")}:
          </span>
          <input
            type="text"
            value={editRoot}
            onChange={(e) => setEditRoot(e.target.value)}
            placeholder="D:\\AgenticTools"
            className="flex-1 rounded-lg border border-border bg-muted/50 px-3 py-1.5 font-mono text-[13px] transition-colors focus:border-primary focus:outline-none focus:ring-0"
            onBlur={(e) => {
              void persistInstallRoot(e.target.value);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                (e.target as HTMLInputElement).blur();
              }
            }}
          />
          <Button
            variant="ghost"
            size="icon"
            title={t("tools.browseFolder", "浏览文件夹")}
            className="h-8 w-8 text-muted-foreground hover:text-foreground"
            onClick={() => {
              import("@tauri-apps/plugin-dialog").then(({ open }) => {
                open({ directory: true, multiple: false }).then((result) => {
                  if (result && typeof result === "string") {
                    setEditRoot(result);
                    void persistInstallRoot(result);
                  }
                });
              });
            }}
          >
            <FolderOpen className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  );
}
