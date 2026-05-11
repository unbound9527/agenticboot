import { useState, useCallback, useEffect, useRef } from "react";
import {
  Loader2,
  CheckCircle,
  AlertTriangle,
  ExternalLink,
  FolderOpen,
  RefreshCw,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { InstallProgress } from "@/components/tools/InstallProgress";
import { NetworkHelpDialog } from "@/components/tools/NetworkHelpDialog";
import { ToolIcon } from "@/components/tools/ToolIcon";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Card } from "@/components/ui/card";
import {
  useCheckNetwork,
  useResolveInstallPlan,
  useExecuteInstallPlan,
} from "@/hooks/useTools";
import { useInstallProgress } from "@/hooks/useInstallProgress";
import { toolsApi } from "@/lib/api/tools";
import type { DetectResult, InstallPlan } from "@/types/tools";

const AVAILABLE_TOOLS: { id: string; name: string; description: string }[] = [
  {
    id: "claude-code-cli",
    name: "Claude Code (CLI)",
    description: "Anthropic 官方 CLI AI 编程助手",
  },
  {
    id: "claude-code-desktop",
    name: "Claude Code (桌面版)",
    description: "Claude Code 桌面应用",
  },
  {
    id: "codex-cli",
    name: "Codex (CLI)",
    description: "OpenAI 官方 CLI 编程助手",
  },
  {
    id: "codex-desktop",
    name: "Codex (桌面版)",
    description: "Codex 桌面应用",
  },
  {
    id: "gemini-cli",
    name: "Gemini CLI",
    description: "Google Gemini CLI 编程助手",
  },
  {
    id: "opencode-cli",
    name: "OpenCode (CLI)",
    description: "开源 AI 编程工具",
  },
  {
    id: "opencode-desktop",
    name: "OpenCode (桌面版)",
    description: "OpenCode 桌面应用",
  },
  { id: "openclaw", name: "OpenClaw", description: "可编程 AI 编码引擎" },
  {
    id: "hermes",
    name: "Hermes (Web UI)",
    description: "多提供商 AI 编程助手，Web UI 交互",
  },
];

const DEFAULT_ROOT = "D:\\AgenticTools";
const INSTALL_ROOT_BOOTSTRAP_TIMEOUT_MS = 500;

interface WizardProps {
  onComplete: () => void;
  initialSelectedToolIds?: string[];
  forceDetectionRefreshToken?: number;
}

function buildSelectionFromDetection(
  ids: string[],
  results: DetectResult[],
  requestedToolIds?: string[],
) {
  const detected = new Set<string>();
  results.forEach((result, index) => {
    if (result.installed) detected.add(ids[index]);
  });

  const available = ids.filter((id) => !detected.has(id));
  const selected =
    requestedToolIds && requestedToolIds.length > 0
      ? available.filter((id) => new Set(requestedToolIds).has(id))
      : available;

  return {
    detected,
    selected: new Set(selected),
  };
}

function buildPendingInstallPlan(toolIds: string[]): InstallPlan {
  const selectedIds = new Set(toolIds);

  return {
    steps: AVAILABLE_TOOLS.filter((tool) => selectedIds.has(tool.id)).map(
      (tool) => ({
        toolId: tool.id,
        toolName: tool.name,
        category: "tool",
        reason: "selected",
        isInstalled: false,
      }),
    ),
  };
}

export function Wizard({
  onComplete,
  initialSelectedToolIds,
  forceDetectionRefreshToken = 0,
}: WizardProps) {
  const { t } = useTranslation();
  const [rootPath, setRootPath] = useState(DEFAULT_ROOT);
  const [isInstallRootReady, setIsInstallRootReady] = useState(false);
  const [selectedTools, setSelectedTools] = useState<Set<string>>(new Set());
  const [installPlan, setInstallPlan] = useState<InstallPlan | null>(null);
  const [started, setStarted] = useState(false);
  const [helpOpen, setHelpOpen] = useState(false);
  const [installedIds, setInstalledIds] = useState<Set<string>>(new Set());
  const [isDetectingTools, setIsDetectingTools] = useState(true);
  const detectionRequestIdRef = useRef(0);
  const activeRootPathRef = useRef(rootPath);
  const rootPathDirtyRef = useRef(false);
  const lastAppliedForceDetectionRefreshTokenRef = useRef(0);

  useEffect(() => {
    let cancelled = false;
    const fallbackTimer = setTimeout(() => {
      if (!cancelled) {
        setIsInstallRootReady(true);
      }
    }, INSTALL_ROOT_BOOTSTRAP_TIMEOUT_MS);

    toolsApi.getInstallRoot()
      .then((savedRoot) => {
        if (cancelled) {
          return;
        }

        if (savedRoot) {
          if (!rootPathDirtyRef.current) {
            setRootPath(savedRoot);
          }
        }
        setIsInstallRootReady(true);
      })
      .catch(() => {
        if (!cancelled) {
          setIsInstallRootReady(true);
        }
      })
      .finally(() => {
        clearTimeout(fallbackTimer);
      });

    return () => {
      cancelled = true;
      clearTimeout(fallbackTimer);
    };
  }, []);

  const refreshDetectedTools = useCallback(
    (forceRefresh = false) => {
      const ids = AVAILABLE_TOOLS.map((tool) => tool.id);
      const requestId = ++detectionRequestIdRef.current;
      setIsDetectingTools(true);
      const detectPromise = toolsApi.detectTools(ids, rootPath, forceRefresh);

      return detectPromise
        .then((results) => {
          if (requestId !== detectionRequestIdRef.current) {
            return;
          }

          const next = buildSelectionFromDetection(
            ids,
            results,
            initialSelectedToolIds,
          );
          setInstalledIds(next.detected);
          setSelectedTools(next.selected);
        })
        .catch((error) => {
          if (requestId !== detectionRequestIdRef.current) {
            return;
          }

          throw error;
        })
        .finally(() => {
          if (requestId === detectionRequestIdRef.current) {
            setIsDetectingTools(false);
          }
        });
    },
    [initialSelectedToolIds, rootPath],
  );

  useEffect(() => {
    if (!isInstallRootReady) {
      activeRootPathRef.current = rootPath;
      return;
    }

    if (activeRootPathRef.current !== rootPath) {
      activeRootPathRef.current = rootPath;
      detectionRequestIdRef.current += 1;
      setIsDetectingTools(true);
      return;
    }

    activeRootPathRef.current = rootPath;
  }, [isInstallRootReady, rootPath]);

  useEffect(() => {
    if (!isInstallRootReady) {
      return;
    }

    let cancelled = false;
    const shouldForceRefresh =
      forceDetectionRefreshToken > lastAppliedForceDetectionRefreshTokenRef.current;
    if (shouldForceRefresh) {
      lastAppliedForceDetectionRefreshTokenRef.current =
        forceDetectionRefreshToken;
    }
    const timer = setTimeout(() => {
      refreshDetectedTools(shouldForceRefresh).catch(() => {
        if (!cancelled) {
          setInstalledIds(new Set());
          setSelectedTools(new Set());
          setIsDetectingTools(false);
        }
      });
    }, 500);

    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [forceDetectionRefreshToken, isInstallRootReady, refreshDetectedTools]);

  const {
    data: netStatus,
    isLoading: netLoading,
    isFetching: netFetching,
    isError: netError,
    refetch: retryNet,
  } = useCheckNetwork();
  const resolvePlan = useResolveInstallPlan();
  const executePlan = useExecuteInstallPlan();
  const { resetProgress } = useInstallProgress();

  const netOk =
    !netLoading && !netFetching && !netError && !netStatus?.errorMessage;

  const handleStartInstall = useCallback(() => {
    const toolIds = [...selectedTools];
    if (toolIds.length === 0) {
      toast.error(t("tools.noToolsSelected", "请至少选择一个工具"));
      return;
    }

    setInstallPlan(buildPendingInstallPlan(toolIds));
    resetProgress();
    setStarted(true);

    resolvePlan.mutate(
      { toolIds, installRoot: rootPath || undefined },
      {
        onSuccess: (plan) => {
          setInstallPlan(plan);
          executePlan.mutate(
            { plan, rootPath },
            {
              onError: (err) => {
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
          setStarted(false);
          setInstallPlan(null);
          toast.error(
            t("tools.resolvePlanFailed", "解析安装计划失败: {{error}}", {
              error: String(err),
            }),
          );
        },
      },
    );
  }, [executePlan, resetProgress, resolvePlan, rootPath, selectedTools, t]);

  const toggleTool = useCallback(
    (id: string) => {
      if (isDetectingTools || installedIds.has(id)) {
        return;
      }

      setSelectedTools((prev) => {
        const next = new Set(prev);
        if (next.has(id)) next.delete(id);
        else next.add(id);
        return next;
      });
    },
    [installedIds, isDetectingTools],
  );

  const availableIds = AVAILABLE_TOOLS.map((tool) => tool.id).filter(
    (id) => !installedIds.has(id),
  );

  const toggleAll = useCallback(() => {
    if (isDetectingTools) {
      return;
    }

    if (selectedTools.size === availableIds.length) {
      setSelectedTools(new Set());
    } else {
      setSelectedTools(new Set(availableIds));
    }
  }, [availableIds, isDetectingTools, selectedTools.size]);

  if (started && installPlan) {
    return (
      <div className="px-6 py-6">
        <div className="mb-8 text-center">
          <h1 className="text-2xl font-bold">
            {t("tools.wizardInstall", "安装中")}
          </h1>
        </div>
        <InstallProgress installPlan={installPlan} onComplete={onComplete} />
      </div>
    );
  }

  return (
    <div className="px-6 py-6">
      <div className="mb-8 text-center">
        <h1 className="text-2xl font-bold">
          {t("tools.wizardTitle", "装机向导")}
        </h1>
      </div>

      <div className="space-y-8">
        <section className="space-y-3 rounded-lg border p-4">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">
              {t("tools.wizardTools", "选择工具")}
            </span>
            <div className="flex items-center gap-2">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  refreshDetectedTools(true).catch(() => {});
                }}
                className="text-xs"
                disabled={isDetectingTools}
              >
                <RefreshCw className="mr-1 h-3 w-3" />
                {t("tools.refreshDetection", "重新检测")}
              </Button>
              <Button
                variant="link"
                size="sm"
                onClick={toggleAll}
                className="text-xs"
                disabled={isDetectingTools}
              >
                {selectedTools.size === availableIds.length
                  ? t("tools.none", "全部取消")
                  : t("tools.all", "全部勾选")}
              </Button>
            </div>
          </div>

          {isDetectingTools && (
            <p className="text-xs text-muted-foreground">
              {t("tools.detectingInstalled", "正在识别已安装工具...")}
            </p>
          )}

          <div className="space-y-1">
            {AVAILABLE_TOOLS.map((tool) => {
              const isInstalled = installedIds.has(tool.id);
              const isDisabled = isInstalled || isDetectingTools;

              return (
                <Card
                  key={tool.id}
                  className={`flex items-center gap-3 p-3 transition-colors ${
                    isInstalled
                      ? "cursor-default opacity-60"
                      : isDetectingTools
                        ? "cursor-wait opacity-80"
                        : "cursor-pointer hover:border-blue-500/50"
                  }`}
                  onClick={() => toggleTool(tool.id)}
                >
                  {isInstalled ? (
                    <CheckCircle className="h-4 w-4 flex-shrink-0 text-green-500" />
                  ) : (
                    <Checkbox
                      checked={selectedTools.has(tool.id)}
                      disabled={isDisabled}
                      onCheckedChange={() => toggleTool(tool.id)}
                    />
                  )}
                  <ToolIcon
                    toolId={tool.id}
                    size={20}
                    className="flex-shrink-0"
                  />
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <Label
                        className={`text-sm font-medium ${isDisabled ? "" : "cursor-pointer"}`}
                      >
                        {tool.name}
                      </Label>
                      {isInstalled && (
                        <Badge
                          variant="outline"
                          className="text-xs text-green-600 dark:text-green-400"
                        >
                          已安装
                        </Badge>
                      )}
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {tool.description}
                    </p>
                  </div>
                </Card>
              );
            })}
          </div>

          <p className="pt-1 text-center text-xs text-muted-foreground">
            {t(
              "tools.autoDepsNote",
              "所需依赖，如 Node.js 和 Git，将在安装过程中自动配置，无需手动处理",
            )}
          </p>
        </section>

        <section className="space-y-3 rounded-lg border p-4">
          <Label htmlFor="install-root" className="text-sm font-medium">
            {t("tools.installRoot", "安装根目录")}
          </Label>
          <div className="flex gap-2">
            <Input
              id="install-root"
              value={rootPath}
              onChange={(e) => {
                rootPathDirtyRef.current = true;
                setRootPath(e.target.value);
              }}
              placeholder={DEFAULT_ROOT}
              className="font-mono text-sm"
            />
            <Button
              variant="outline"
              size="icon"
              title={t("tools.browseFolder", "浏览文件夹")}
              onClick={() => {
                import("@tauri-apps/plugin-dialog").then(({ open }) => {
                  open({ directory: true, multiple: false }).then((result) => {
                    if (result && typeof result === "string") {
                      rootPathDirtyRef.current = true;
                      setRootPath(result);
                    }
                  });
                });
              }}
            >
              <FolderOpen className="h-4 w-4" />
            </Button>
          </div>
          <p className="text-xs text-muted-foreground">
            {t("tools.installRootHint", "所有工具将安装到此目录下的子目录中")}
          </p>
        </section>

        <section className="rounded-lg border p-4">
          <div className="flex items-center gap-4">
            {(netLoading || netFetching) && (
              <Loader2 className="h-4 w-4 flex-shrink-0 animate-spin text-blue-500" />
            )}
            {!netLoading && !netFetching && netOk && (
              <CheckCircle className="h-4 w-4 flex-shrink-0 text-green-500" />
            )}
            {!netLoading &&
              !netFetching &&
              (netError || netStatus?.errorMessage) && (
                <AlertTriangle className="h-4 w-4 flex-shrink-0 text-amber-500" />
              )}

            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-3 text-xs">
                <span className="text-muted-foreground">连通性</span>
                {netLoading || netFetching ? (
                  <span className="text-muted-foreground">检测中...</span>
                ) : (
                  <>
                    <span
                      className={
                        netStatus?.githubReachable
                          ? "text-green-600"
                          : "text-red-500"
                      }
                    >
                      GitHub {netStatus?.githubReachable ? "✓" : "×"}
                    </span>
                    <span
                      className={
                        netStatus?.npmReachable
                          ? "text-green-600"
                          : "text-red-500"
                      }
                    >
                      npm {netStatus?.npmReachable ? "✓" : "×"}
                    </span>
                    <span
                      className={
                        netStatus?.youtubeReachable
                          ? "text-green-600"
                          : "text-muted-foreground"
                      }
                    >
                      YouTube {netStatus?.youtubeReachable ? "✓" : "×"}
                    </span>
                  </>
                )}
              </div>
              {netStatus?.errorMessage && (
                <p className="mt-1 text-xs text-amber-600 dark:text-amber-400">
                  {netStatus.errorMessage}
                </p>
              )}
            </div>

            <Button
              variant="ghost"
              size="sm"
              onClick={() => retryNet()}
              title="刷新"
            >
              <RefreshCw className="h-3 w-3" />
            </Button>
          </div>

          {!netLoading && !netFetching && !netOk && (
            <button
              onClick={() => setHelpOpen(true)}
              className="mt-3 inline-flex items-center gap-1 text-xs text-blue-500 hover:text-blue-600 hover:underline"
            >
              <ExternalLink className="h-3 w-3" />
              网络不通？点击查看解决方法
            </button>
          )}
        </section>

        <div className="flex justify-center gap-4 pt-2">
          <Button variant="outline" size="lg" onClick={onComplete}>
            {t("tools.skipForNow", "跳过")}
          </Button>
          <Button
            size="lg"
            onClick={handleStartInstall}
            disabled={
              isDetectingTools || resolvePlan.isPending || !rootPath.trim()
            }
            className="px-12"
          >
            {resolvePlan.isPending
              ? t("tools.resolving", "解析中...")
              : t("tools.startInstall", "开始安装")}
          </Button>
        </div>
      </div>

      <NetworkHelpDialog open={helpOpen} onOpenChange={setHelpOpen} />
    </div>
  );
}
