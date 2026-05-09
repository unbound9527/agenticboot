// 安装进度展示组件

import { CheckCircle, Loader2, XCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { useInstallProgress } from "@/hooks/useInstallProgress";
import { useTranslation } from "react-i18next";
import type { InstallPlan } from "@/types/tools";

interface InstallProgressProps {
  installPlan: InstallPlan;
  onComplete: () => void;
}

export function InstallProgress({
  installPlan,
  onComplete,
}: InstallProgressProps) {
  const { t } = useTranslation();
  const { getToolProgress, allComplete, hasErrors } = useInstallProgress();

  const overallPercent = Math.round(
    (installPlan.steps.filter(
      (s) =>
        s.isInstalled ||
        getToolProgress(s.toolId)?.phase === "complete" ||
        getToolProgress(s.toolId)?.phase === "skipped",
    ).length /
      installPlan.steps.length) *
      100,
  );

  const getPhaseLabel = (phase: string | undefined): string => {
    switch (phase) {
      case "downloading":
        return "> 正在下载...";
      case "extracting":
        return "> 正在解压...";
      case "installing":
        return "> 正在安装...";
      case "configuring":
        return "> 正在配置...";
      case "complete":
        return "> [OK] 安装完成";
      case "error":
        return "> [ERROR] 安装失败";
      case "skipped":
        return "> [SKIP] 已安装，跳过";
      default:
        return "> 等待中...";
    }
  };

  return (
    <div className="flex flex-col space-y-6 py-8">
      {/* Terminal-style card */}
      <div className="rounded-lg border bg-background overflow-hidden shadow-sm">
        {/* Terminal header */}
        <div className="bg-muted border-b px-4 py-3 flex justify-between items-center font-mono">
          <span className="text-sm font-medium">AgenticBoot UI v1.0</span>
          <span className="text-sm text-muted-foreground">_ □ ×</span>
        </div>

        {/* Terminal content */}
        <div className="p-6 font-mono text-sm space-y-4">
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
                  {/* Status icon */}
                  {isComplete && (
                    <CheckCircle className="h-4 w-4 flex-shrink-0 text-green-500" />
                  )}
                  {isError && (
                    <XCircle className="h-4 w-4 flex-shrink-0 text-red-500" />
                  )}
                  {isActive && (
                    <Loader2 className="h-4 w-4 flex-shrink-0 animate-spin text-blue-500" />
                  )}

                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span
                        className={`font-medium ${isError ? "line-through text-red-500" : ""}`}
                      >
                        {step.toolName}
                      </span>
                      {isError && progress?.message && (
                        <span className="text-red-500 text-xs">
                          - {progress.message}
                        </span>
                      )}
                    </div>
                    <p
                      className={`text-xs ${isActive ? "text-muted-foreground" : isError ? "text-red-400" : "text-green-600"}`}
                    >
                      {getPhaseLabel(progress?.phase)}
                    </p>
                    {progress?.message && (
                      <p className="mt-1 text-xs text-muted-foreground">
                        {progress.message}
                      </p>
                    )}
                    {/* Step progress bar */}
                    {isActive && (
                      <Progress
                        value={progress?.percent ?? 0}
                        className="h-1 mt-1"
                      />
                    )}
                  </div>
                </div>
              );
            })}
          </div>

          {/* Overall progress */}
          <div className="border-t pt-4 space-y-1">
            <div className="flex justify-between text-xs">
              <span>总进度:</span>
              <span>{overallPercent}%</span>
            </div>
            <Progress value={overallPercent} className="h-3" />
            <p className="text-xs text-muted-foreground">
              {Math.round((overallPercent * installPlan.steps.length) / 100)} /{" "}
              {installPlan.steps.length} 个工具
            </p>
          </div>
        </div>
      </div>

      {/* Completion and error handling */}
      {allComplete && !hasErrors && (
        <div className="text-center space-y-4 pt-4">
          <div className="inline-flex items-center justify-center w-20 h-20 bg-green-500 text-white rounded-full">
            <CheckCircle className="h-10 w-10" />
          </div>
          <p className="text-xl font-semibold">
            {t("tools.installComplete", "安装完成！")}
          </p>
          <Button variant="default" onClick={onComplete}>
            {t("tools.enterManager", "进入管理")}
          </Button>
        </div>
      )}

      {hasErrors && (
        <div className="text-center space-y-3 pt-4">
          <div className="inline-flex items-center justify-center w-20 h-20 bg-red-500 text-white rounded-full">
            <XCircle className="h-10 w-10" />
          </div>
          <p className="text-sm text-muted-foreground">
            {t("tools.installPartial", "部分工具安装失败，可稍后重试")}
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
