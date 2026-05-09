import { useState, useCallback, useEffect } from "react";
import { FolderOpen, RefreshCw, Settings } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { ToolCard } from "@/components/tools/ToolCard";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useInstallProgress } from "@/hooks/useInstallProgress";
import {
  useInstalledTools,
  useInstallRoot,
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
}

function toExternalInstalledTool(
  meta: ToolMeta,
  detect: DetectResult,
  installRoot: string | null | undefined,
): InstalledTool {
  return {
    id: meta.id,
    name: meta.name,
    version: detect.version,
    installPath: detect.installPath ?? "",
    installRoot: installRoot ?? "",
    category: "tool",
    status: "installed",
  };
}

export function Manager({ onInstallMore }: ManagerProps) {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState("installed");
  const [editRoot, setEditRoot] = useState("");
  const [detectedTools, setDetectedTools] = useState<
    Record<string, DetectResult>
  >({});

  const { data: installedTools = [] } = useInstalledTools();
  const { data: installRoot } = useInstallRoot();
  const { data: updates = [] } = useToolUpdates();
  const uninstallTool = useUninstallTool();
  const { getToolProgress } = useInstallProgress();
  const managedInstalledTools = installedTools.filter(
    (tool) => tool.status === "installed",
  );

  useEffect(() => {
    setEditRoot(installRoot ?? "");
  }, [installRoot]);

  const refreshDetectedTools = useCallback((forceRefresh = false) => {
    const ids = ALL_TOOLS_META.map((tool) => tool.id);
    const detectPromise = forceRefresh
      ? toolsApi.detectTools(ids, installRoot ?? undefined, true)
      : toolsApi.detectTools(ids, installRoot ?? undefined);
    return detectPromise.then((results) => {
      const next: Record<string, DetectResult> = {};
      results.forEach((result, index) => {
        next[ids[index]] = result;
      });
      setDetectedTools(next);
    });
  }, [installRoot]);

  useEffect(() => {
    let cancelled = false;
    const timer = setTimeout(() => {
      refreshDetectedTools(false).catch(() => {
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
        toExternalInstalledTool(meta, detectedTools[meta.id], installRoot),
      ),
  ];

  const notInstalled = ALL_TOOLS_META.filter(
    (meta) => !installedIds.has(meta.id),
  );

  const handleUninstall = useCallback(
    (toolId: string, rootPath: string) => {
      uninstallTool.mutate(
        { toolId, rootPath },
        {
          onSuccess: () => {
            toast.success(t("tools.uninstalled", "卸载成功"));
          },
          onError: (err) => {
            toast.error(
              t("tools.uninstallFailed", "卸载失败: {{error}}", {
                error: String(err),
              }),
            );
          },
        },
      );
    },
    [uninstallTool, t],
  );

  const handleSingleInstall = useCallback(
    (_toolId: string) => {
      onInstallMore?.();
    },
    [onInstallMore],
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
          >
            <RefreshCw className="mr-1.5 h-3 w-3" />
            {t("tools.refreshDetection", "重新检测")}
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => {
              if (updates.length > 0) {
                toast.info(
                  t("tools.updatesAvailable", "{{count}} 个工具可更新", {
                    count: updates.length,
                  }),
                );
              } else {
                toast.success(t("tools.allUpToDate", "所有工具均为最新版本"));
              }
            }}
            className="text-[13px]"
          >
            <RefreshCw className="mr-1.5 h-3 w-3" />
            {t("tools.checkUpdates", "检查更新")}
          </Button>
        </div>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="w-full mb-4">
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
              <Button variant="link" onClick={onInstallMore} className="mt-2 text-[13px]">
                {t("tools.goInstall", "去安装")}
              </Button>
            </div>
          )}
          {mergedInstalledTools.map((tool) => {
            const isManagedRecord = managedInstalledIds.has(tool.id);

            return (
              <ToolCard
                key={tool.id}
                tool={tool}
                variant="installed"
                progress={getToolProgress(tool.id)}
                onUninstall={
                  isManagedRecord
                    ? () => handleUninstall(tool.id, tool.installRoot)
                    : undefined
                }
                onUpdate={
                  updates.find((update) => update.toolId === tool.id)
                    ? () => handleSingleInstall(tool.id)
                    : undefined
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
              onInstall={() => handleSingleInstall(tool.id)}
            />
          ))}
        </TabsContent>
      </Tabs>

      <div className="mt-6 pt-5 border-t border-border/50">
        <div className="flex items-center gap-3 text-[13px] text-muted-foreground">
          <Settings className="h-4 w-4 flex-shrink-0" />
          <span className="flex-shrink-0 font-medium">
            {t("tools.installRoot", "安装根目录")}:
          </span>
          <input
            type="text"
            value={editRoot}
            onChange={(e) => setEditRoot(e.target.value)}
            placeholder="D:\\AgenticBoot"
            className="flex-1 rounded-lg border border-border bg-muted/50 px-3 py-1.5 font-mono text-[13px] focus:border-primary focus:ring-0 focus:outline-none transition-colors"
            onBlur={(e) => {
              const newRoot = e.target.value.trim();
              if (newRoot && newRoot !== installRoot) {
                toolsApi.setInstallRoot(newRoot).catch(() => {});
              }
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") (e.target as HTMLInputElement).blur();
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
                    toolsApi.setInstallRoot(result).catch(() => {});
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
