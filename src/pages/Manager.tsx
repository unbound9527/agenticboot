import { useState, useCallback, useEffect } from "react";
import { FolderOpen, RefreshCw, Settings } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { useQueryClient } from "@tanstack/react-query";
import { InstallConsole } from "@/components/tools/InstallConsole";
import { ToolCard } from "@/components/tools/ToolCard";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useInstallProgress } from "@/hooks/useInstallProgress";
import { useInstallSessions } from "@/hooks/useInstallSessions";
import {
  useExecuteInstallPlan,
  useInstalledTools,
  useInstallRoot,
  useResolveInstallPlan,
  useToolUpdates,
  useUninstallTool,
} from "@/hooks/useTools";
import { toolsApi } from "@/lib/api/tools";
import type { DetectResult, InstalledTool, ToolMeta } from "@/types/tools";

const ALL_TOOLS_META: ToolMeta[] = [
  {
    id: "claude-code-cli",
    name: "Claude Code (CLI)",
    description: "Anthropic 官方 CLI AI 编程助手",
    icon: "claude",
    category: "ai-cli",
  },
  {
    id: "claude-code-desktop",
    name: "Claude Code (桌面版)",
    description: "Claude Code 桌面应用",
    icon: "claude",
    category: "ai-cli",
  },
  {
    id: "codex-cli",
    name: "Codex (CLI)",
    description: "OpenAI 官方 CLI 编程助手",
    icon: "codex",
    category: "ai-cli",
  },
  {
    id: "codex-desktop",
    name: "Codex (桌面版)",
    description: "Codex 桌面应用",
    icon: "codex",
    category: "ai-cli",
  },
  {
    id: "gemini-cli",
    name: "Gemini CLI",
    description: "Google Gemini CLI 编程助手",
    icon: "gemini",
    category: "ai-cli",
  },
  {
    id: "opencode-cli",
    name: "OpenCode (CLI)",
    description: "开源 AI 编程工具",
    icon: "opencode",
    category: "ai-cli",
  },
  {
    id: "opencode-desktop",
    name: "OpenCode (桌面版)",
    description: "OpenCode 桌面应用",
    icon: "opencode",
    category: "ai-cli",
  },
  {
    id: "openclaw",
    name: "OpenClaw",
    description: "可编程 AI 编码引擎",
    icon: "openclaw",
    category: "ai-cli",
  },
  {
    id: "hermes",
    name: "Hermes (Web UI)",
    description: "多提供商 AI 编程助手，Web UI 交互",
    icon: "hermes",
    category: "ai-cli",
  },
];

interface ManagerProps {
  onInstallMore?: () => void;
  onToolStateChanged?: () => void;
}

function toExternalInstalledTool(
  meta: ToolMeta,
  detect: DetectResult,
  installRoot: string,
): InstalledTool {
  return {
    id: meta.id,
    name: meta.name,
    version: detect.version,
    installPath: detect.installPath ?? "",
    installRoot,
    category: "tool",
    status: "installed",
  };
}

function buildPendingInstallProgress(tool: ToolMeta) {
  return {
    toolId: tool.id,
    toolName: tool.name,
    phase: "starting" as const,
    percent: 0,
    message: "Preparing installation...",
  };
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
  const [uninstallingToolId, setUninstallingToolId] = useState<string | null>(null);
  const [pendingInstallToolId, setPendingInstallToolId] = useState<string | null>(null);

  const { data: installedTools = [] } = useInstalledTools();
  const { data: installRoot } = useInstallRoot();
  const { data: updates = [] } = useToolUpdates();
  const uninstallTool = useUninstallTool();
  const resolvePlan = useResolveInstallPlan();
  const executePlan = useExecuteInstallPlan();
  const { getToolProgress, resetProgress } = useInstallProgress();
  const installSessions = useInstallSessions();
  const managedInstalledTools = installedTools.filter(
    (tool) => tool.status === "installed",
  );
  const effectiveInstallRoot = activeInstallRoot || installRoot || "";
  const selectedConsoleSession = openConsoleToolId
    ? installSessions.get(openConsoleToolId) ?? null
    : null;

  useEffect(() => {
    const nextRoot = installRoot ?? "";
    setEditRoot(nextRoot);
    setActiveInstallRoot(nextRoot);
  }, [installRoot]);

  const refreshDetectedTools = useCallback(
    (forceRefresh = false) => {
      const ids = ALL_TOOLS_META.map((tool) => tool.id);
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
    [effectiveInstallRoot],
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
    ...ALL_TOOLS_META.filter((meta) => !managedInstalledIds.has(meta.id))
      .filter((meta) => detectedTools[meta.id]?.installed)
      .map((meta) =>
        toExternalInstalledTool(meta, detectedTools[meta.id], effectiveInstallRoot),
      ),
  ];

  const notInstalled = ALL_TOOLS_META.filter(
    (meta) => !installedIds.has(meta.id),
  );

  const handleUninstall = useCallback(
    (toolId: string, rootPath: string) => {
      setUninstallingToolId(toolId);
      uninstallTool.mutate(
        { toolId, rootPath },
        {
          onSuccess: () => {
            toast.success(t("tools.uninstalled", "卸载成功"));
            onToolStateChanged?.();
            refreshDetectedTools(true).catch(() => {});
          },
          onError: (err) => {
            setPendingInstallToolId(null);
            toast.error(
              t("tools.uninstallFailed", "卸载失败: {{error}}", {
                error: String(err),
              }),
            );
          },
          onSettled: () => {
            setUninstallingToolId(null);
          },
        },
      );
    },
    [onToolStateChanged, refreshDetectedTools, uninstallTool, t],
  );

  const handleSingleInstall = useCallback(
    (toolId: string, rootPath?: string) => {
      const resolvedRoot = (rootPath ?? effectiveInstallRoot).trim();
      if (!resolvedRoot) {
        toast.error(t("tools.installRootRequired", "请先设置安装根目录"));
        return;
      }

      setPendingInstallToolId(toolId);
      resolvePlan.mutate(
        { toolIds: [toolId], installRoot: resolvedRoot },
        {
          onSuccess: (plan) => {
            resetProgress();
            executePlan.mutate(
              { plan, rootPath: resolvedRoot },
              {
                onSuccess: () => {
                  toast.success(t("tools.installStarted", "已开始安装"));
                },
                onError: (err) => {
                  setPendingInstallToolId(null);
                  toast.error(
                    t("tools.installFailed", "安装失败: {{error}}", {
                      error: String(err),
                    }),
                  );
                },
              },
            );
          },
          onError: (err) => {
            toast.error(
              t("tools.resolvePlanFailed", "解析安装计划失败: {{error}}", {
                error: String(err),
              }),
            );
          },
        },
      );
    },
    [effectiveInstallRoot, executePlan, resetProgress, resolvePlan, t],
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
              queryClient.invalidateQueries({ queryKey: ["tool-updates"] }).finally(() => {
                setIsCheckingUpdates(false);
                if (updates.length > 0) {
                  toast.info(
                    t("tools.updatesAvailable", "{{count}} 个工具可更新", {
                      count: updates.length,
                    }),
                  );
                } else {
                  toast.success(t("tools.allUpToDate", "所有工具均为最新版本"));
                }
              });
            }}
            className="text-[13px]"
          >
            <RefreshCw className={`mr-1.5 h-3 w-3 ${isCheckingUpdates ? "animate-spin" : ""}`} />
            {t("tools.checkUpdates", "检查更新")}
          </Button>
        </div>
      </div>

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
            const canUninstall = Boolean(tool.installPath?.trim());

            return (
              <ToolCard
                key={tool.id}
                tool={tool}
                variant="installed"
                isUninstalling={uninstallingToolId === tool.id}
                progress={getToolProgress(tool.id)}
                installSession={installSessions.get(tool.id) ?? null}
                onUninstall={
                  canUninstall
                    ? () => handleUninstall(tool.id, tool.installRoot)
                    : undefined
                }
                onUpdate={
                  updates.find((update) => update.toolId === tool.id)
                    ? () => handleSingleInstall(tool.id, tool.installRoot)
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
              progress={
                getToolProgress(tool.id) ??
                (pendingInstallToolId === tool.id
                  ? buildPendingInstallProgress(tool)
                  : null)
              }
              installSession={installSessions.get(tool.id) ?? null}
              onInstall={() => {
                handleSingleInstall(tool.id, effectiveInstallRoot);
                setOpenConsoleToolId(tool.id);
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

      {selectedConsoleSession && (
        <div className="mt-4">
          <InstallConsole session={selectedConsoleSession} />
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
