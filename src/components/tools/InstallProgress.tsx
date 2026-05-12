import { CheckCircle, Loader2, XCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { InstallConsole } from "@/components/tools/InstallConsole";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { useInstallProgress } from "@/hooks/useInstallProgress";
import { useInstallSessions } from "@/hooks/useInstallSessions";
import type { InstallPlan, ToolInstallSession } from "@/types/tools";

interface InstallProgressProps {
  installPlan: InstallPlan;
  onComplete: () => void;
  installSession?: ToolInstallSession | null;
}

function getPhaseLabel(
  t: ReturnType<typeof useTranslation>["t"],
  phase: string | undefined,
): string {
  switch (phase) {
    case "downloading":
      return `> ${t("tools.phaseDownloading", "下载中...")}`;
    case "extracting":
      return `> ${t("tools.phaseExtracting", "解压中...")}`;
    case "installing":
      return `> ${t("tools.phaseInstalling", "安装中...")}`;
    case "configuring":
      return `> ${t("tools.phaseConfiguring", "配置中...")}`;
    case "complete":
      return `> [OK] ${t("tools.installComplete", "安装完成")}`;
    case "error":
      return `> [ERROR] ${t("tools.phaseError", "失败")}`;
    case "skipped":
      return `> [SKIP] ${t("tools.phaseSkipped", "已安装，跳过")}`;
    default:
      return `> ${t("tools.phasePending", "等待中...")}`;
  }
}

const GENERIC_PROGRESS_MESSAGES = new Set([
  "working...",
  "starting...",
  "downloading...",
  "extracting...",
  "installing...",
  "configuring...",
  "please wait...",
]);

function isGenericProgressMessage(message: string | undefined): boolean {
  if (!message) {
    return true;
  }

  const normalized = message.trim().toLowerCase();
  return GENERIC_PROGRESS_MESSAGES.has(normalized);
}

export function InstallProgress({
  installPlan,
  onComplete,
  installSession = null,
}: InstallProgressProps) {
  const { t } = useTranslation();
  const { getToolProgress, allComplete, hasErrors } = useInstallProgress();
  const installSessions = useInstallSessions();
  const activeToolId =
    installPlan.steps.find((step) => {
      const progress = getToolProgress(step.toolId);
      return (
        progress && !["complete", "error", "skipped"].includes(progress.phase)
      );
    })?.toolId ?? null;
  const latestPlanSession =
    installPlan.steps
      .map((step) => installSessions.get(step.toolId) ?? null)
      .filter((session): session is ToolInstallSession => session !== null)
      .sort((left, right) => {
        const leftTime = Date.parse(left.endedAt ?? left.startedAt);
        const rightTime = Date.parse(right.endedAt ?? right.startedAt);
        return rightTime - leftTime;
      })[0] ?? null;
  const activeSession =
    installSession ??
    (activeToolId
      ? installSessions.get(activeToolId) ?? null
      : latestPlanSession);
  const activeProgress = activeToolId ? getToolProgress(activeToolId) : null;
  const recentActivity = activeSession?.activity.slice(-3) ?? [];
  const primaryActionText =
    activeToolId && activeProgress
      ? !isGenericProgressMessage(activeProgress.message)
        ? activeProgress.message
        : recentActivity.at(-1)?.line ??
          activeSession?.lastSummary ??
          getPhaseLabel(t, activeProgress.phase).replace(/^>\s*/, "")
      : null;
  const visibleRecentActivity = recentActivity.filter(
    (item, index) =>
      item.line !== primaryActionText || index !== recentActivity.length - 1,
  );
  const overallPercent = Math.round(
    installPlan.steps.reduce((sum, step) => {
      const progress = getToolProgress(step.toolId);
      const stepPercent = step.isInstalled
        ? 100
        : progress?.phase === "complete" || progress?.phase === "skipped"
          ? 100
          : progress?.phase === "error"
            ? progress.percent
            : progress?.percent ?? 0;

      return sum + stepPercent;
    }, 0) / installPlan.steps.length,
  );
  const completedToolCount = installPlan.steps.filter(
    (step) =>
      step.isInstalled ||
      getToolProgress(step.toolId)?.phase === "complete" ||
      getToolProgress(step.toolId)?.phase === "skipped",
  ).length;

  return (
    <div className="flex flex-col space-y-6 py-8">
      <div className="overflow-hidden rounded-lg border bg-background shadow-sm">
        <div className="flex items-center justify-between border-b bg-muted px-4 py-3 font-mono">
          <span className="text-sm font-medium">AgenticBoot UI v1.0</span>
          <span className="text-sm text-muted-foreground">_ [] X</span>
        </div>

        <div className="space-y-4 p-6 font-mono text-sm">
          {activeToolId && activeProgress && (
            <section
              aria-label={t("tools.installProgressCurrentAction", "当前操作")}
              className="rounded-lg border border-blue-500/20 bg-blue-500/5 p-4"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="min-w-0 space-y-1">
                  <p className="text-xs uppercase tracking-[0.2em] text-blue-600">
                    {t("tools.installProgressCurrentAction", "当前操作")}
                  </p>
                  <p className="text-base font-semibold text-foreground">
                    {activeProgress.toolName}
                  </p>
                  <p className="text-sm text-foreground">
                    {primaryActionText}
                  </p>
                </div>
                <span className="text-xs text-muted-foreground">
                  {activeProgress.percent}%
                </span>
              </div>

              {visibleRecentActivity.length > 0 && (
                <div className="mt-3 space-y-1 border-t border-blue-500/10 pt-3 text-xs text-muted-foreground">
                  {visibleRecentActivity.map((item, index) => (
                    <p key={`${item.timestamp}-${index}`} className="truncate">
                      {item.line}
                    </p>
                  ))}
                </div>
              )}
            </section>
          )}

          <div className="space-y-2">
            {installPlan.steps.map((step) => {
              const progress = getToolProgress(step.toolId);
              const isComplete =
                step.isInstalled ||
                progress?.phase === "complete" ||
                progress?.phase === "skipped";
              const isError = progress?.phase === "error";
              const isActive =
                progress &&
                !["complete", "error", "skipped"].includes(progress.phase);

              return (
                <div key={step.toolId} className="flex items-center gap-3">
                  {isComplete && (
                    <CheckCircle className="h-4 w-4 flex-shrink-0 text-green-500" />
                  )}
                  {isError && (
                    <XCircle className="h-4 w-4 flex-shrink-0 text-red-500" />
                  )}
                  {isActive && (
                    <Loader2 className="h-4 w-4 flex-shrink-0 animate-spin text-blue-500" />
                  )}

                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span
                        className={`font-medium ${isError ? "text-red-500 line-through" : ""}`}
                      >
                        {step.toolName}
                      </span>
                      {isError && progress?.message && (
                        <span className="text-xs text-red-500">
                          - {progress.message}
                        </span>
                      )}
                    </div>
                    <p
                      className={`text-xs ${isActive ? "text-muted-foreground" : isError ? "text-red-400" : "text-green-600"}`}
                    >
                      {getPhaseLabel(t, progress?.phase)}
                    </p>
                    {progress?.message && (
                      <p className="mt-1 text-xs text-muted-foreground">
                        {progress.message}
                      </p>
                    )}
                    {isActive && (
                      <Progress value={progress.percent} className="mt-1 h-1" />
                    )}
                  </div>
                </div>
              );
            })}
          </div>

          <div className="space-y-1 border-t pt-4">
            <div className="flex justify-between text-xs">
              <span>{t("tools.installProgressTotal", "总进度")}</span>
              <span>{overallPercent}%</span>
            </div>
            <Progress value={overallPercent} className="h-3" />
            <p className="text-xs text-muted-foreground">
              {t("tools.installProgressToolCount", "{{completed}} / {{total}} 个工具", {
                completed: completedToolCount,
                total: installPlan.steps.length,
              })}
            </p>
          </div>
        </div>
      </div>

      <InstallConsole session={activeSession} />

      {allComplete && !hasErrors && (
        <div className="space-y-4 pt-4 text-center">
          <div className="inline-flex h-20 w-20 items-center justify-center rounded-full bg-green-500 text-white">
            <CheckCircle className="h-10 w-10" />
          </div>
          <p className="text-xl font-semibold">
            {t("tools.installComplete", "安装完成")}
          </p>
          <Button variant="default" onClick={onComplete}>
            {t("tools.enterManager", "进入管理")}
          </Button>
        </div>
      )}

      {hasErrors && (
        <div className="space-y-3 pt-4 text-center">
          <div className="inline-flex h-20 w-20 items-center justify-center rounded-full bg-red-500 text-white">
            <XCircle className="h-10 w-10" />
          </div>
          <p className="text-sm text-muted-foreground">
            {t(
              "tools.installPartial",
              "部分工具安装失败，可稍后重试",
            )}
          </p>
          <div className="flex justify-center gap-3">
            <Button variant="outline" onClick={onComplete}>
              {t("tools.skipForNow", "暂时跳过")}
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
