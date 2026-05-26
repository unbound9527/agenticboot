import {
  ArrowUpCircle,
  Download,
  FolderOpen,
  Loader2,
  Trash2,
  Play,
  Terminal,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { ToolIcon } from "@/components/tools/ToolIcon";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { formatInstalledVersion } from "@/lib/tools/version";
import type {
  InstallProgress,
  InstalledTool,
  ToolInstallSession,
  ToolMeta,
} from "@/types/tools";

type ToolData = InstalledTool | ToolMeta;

interface ToolCardProps {
  tool: ToolData;
  variant: "installed" | "available";
  onInstall?: () => void;
  onUninstall?: () => void;
  onUpdate?: () => void;
  onLaunch?: () => void;
  onOpenFolder?: () => void;
  isUninstalling?: boolean;
  isLaunching?: boolean;
  isUpdating?: boolean;
  isInstalling?: boolean;
  progress?: InstallProgress | null;
  installSession?: ToolInstallSession | null;
  onShowConsole?: () => void;
}

export function ToolCard({
  tool,
  variant,
  onInstall,
  onUninstall,
  onUpdate,
  onLaunch,
  onOpenFolder,
  isUninstalling = false,
  isLaunching = false,
  isUpdating = false,
  isInstalling: isInstallingProp = false,
  progress,
  installSession,
  onShowConsole,
}: ToolCardProps) {
  const { t } = useTranslation();
  const formattedVersion =
    "version" in tool ? formatInstalledVersion(tool.version) : null;
  const hasActiveProgress = Boolean(
    progress && !["complete", "error", "skipped"].includes(progress.phase),
  );
  const isInstalling = isInstallingProp || hasActiveProgress;
  const disableInstalledActions = isInstalling || isUpdating;

  return (
    <div className="claude-card flex items-center gap-4 p-4">
      <ToolIcon toolId={tool.id} size={22} />

      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="truncate text-[14px] font-medium">{tool.name}</span>
          {"category" in tool && (
            <Badge variant="secondary" className="px-1.5 py-0 text-[11px]">
              {tool.category === "dependency"
                ? t("tools.badgeDependency", "依赖")
                : t("tools.badgeTool", "工具")}
            </Badge>
          )}
        </div>
        <p className="mt-0.5 truncate text-[12px] text-muted-foreground">
          {"description" in tool
            ? tool.description
            : tool.installPath || tool.name}
        </p>

        {isInstalling && progress && (
          <>
            <Progress value={progress.percent} className="mt-2 h-1" />
            <p className="mt-1 text-[11px] text-muted-foreground">
              {progress.message}
            </p>
          </>
        )}
      </div>

      <div className="flex flex-shrink-0 items-center gap-1">
        {(installSession?.status === "running" || installSession?.status === "error") && onShowConsole && (
          <Button
            variant="ghost"
            size="icon"
            title={t("tools.console", "控制台")}
            onClick={onShowConsole}
            className="h-8 w-8 text-muted-foreground hover:text-foreground"
          >
            <Terminal className="h-4 w-4" />
          </Button>
        )}

        {variant === "installed" && (
          <>
            <Badge
              variant="secondary"
              className="bg-emerald-100 px-2 py-0.5 text-[11px] text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-400"
            >
              {t("tools.installed", "已安装")}
              {formattedVersion ? ` ${formattedVersion}` : ""}
            </Badge>

            {onOpenFolder && (
              <Button
                variant="ghost"
                size="icon"
                title={t("tools.openFolder", "打开文件夹")}
                onClick={onOpenFolder}
                className="h-8 w-8 text-muted-foreground hover:text-foreground"
              >
                <FolderOpen className="h-4 w-4" />
              </Button>
            )}

            {onLaunch && (
              <Button
                variant="ghost"
                size="icon"
                title={t("tools.launch", "启动")}
                onClick={onLaunch}
                disabled={isLaunching || disableInstalledActions}
                className="h-8 w-8 text-muted-foreground hover:text-emerald-600 disabled:opacity-50"
              >
                {isLaunching ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Play className="h-4 w-4" />
                )}
              </Button>
            )}

            {onUninstall && (
              <Button
                variant="ghost"
                size="icon"
                title={t("tools.uninstall", "卸载")}
                onClick={onUninstall}
                disabled={isUninstalling || disableInstalledActions}
                className="h-8 w-8 text-muted-foreground hover:text-destructive disabled:opacity-50"
              >
                {isUninstalling ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Trash2 className="h-4 w-4" />
                )}
              </Button>
            )}

            {onUpdate && (
              <Button
                variant="ghost"
                size="icon"
                title={t("tools.update", "更新")}
                onClick={onUpdate}
                disabled={disableInstalledActions}
                className="h-8 w-8 text-muted-foreground hover:text-foreground disabled:opacity-50"
              >
                {isUpdating ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <ArrowUpCircle className="h-4 w-4" />
                )}
              </Button>
            )}
          </>
        )}

        {variant === "available" && !isInstalling && onInstall && (
          <Button
            variant="ghost"
            size="icon"
            title={t("tools.install", "安装")}
            onClick={onInstall}
            className="h-8 w-8 text-muted-foreground hover:text-foreground"
          >
            <Download className="h-4 w-4" />
          </Button>
        )}

        {isInstalling && progress && (
          <span className="text-[12px] text-muted-foreground">
            {progress.percent}%
          </span>
        )}
      </div>
    </div>
  );
}
