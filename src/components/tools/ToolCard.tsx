import { Trash2, Download, RefreshCw, Loader2, Zap, FolderOpen } from "lucide-react";
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
  progress,
  installSession,
  onShowConsole,
}: ToolCardProps) {
  const { t } = useTranslation();
  const formattedVersion =
    "version" in tool ? formatInstalledVersion(tool.version) : null;

  const isInstalling =
    progress && !["complete", "error", "skipped"].includes(progress.phase);

  // 用户自行安装的工具（不在管理目录下）不允许卸载
  return (
    <div className="claude-card flex items-center gap-4 p-4">
      <ToolIcon toolId={tool.id} size={22} />

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-[14px] font-medium truncate">{tool.name}</span>
          {"category" in tool && (
            <Badge variant="secondary" className="text-[11px] px-1.5 py-0">
              {tool.category === "dependency"
                ? t("tools.badgeDependency", "依赖")
                : t("tools.badgeTool", "工具")}
            </Badge>
          )}
        </div>
        <p className="text-[12px] text-muted-foreground truncate mt-0.5">
          {"description" in tool
            ? tool.description
            : tool.installPath || tool.name}
        </p>

        {isInstalling && progress && (
          <>
            <Progress value={progress.percent} className="h-1 mt-2" />
            <p className="mt-1 text-[11px] text-muted-foreground">
              {progress.message}
            </p>
          </>
        )}
      </div>

      <div className="flex-shrink-0 flex items-center gap-2">
        {installSession?.status === "running" && onShowConsole && (
          <Button
            variant="ghost"
            size="sm"
            onClick={onShowConsole}
            className="text-[12px]"
          >
            {t("tools.console", "Console")}
          </Button>
        )}

        
        {variant === "installed" && (
          <>
            <Badge
              variant="secondary"
              className="text-[11px] px-2 py-0.5 bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-400"
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
                disabled={isLaunching}
                className="h-8 w-8 text-muted-foreground hover:text-emerald-600 disabled:opacity-50"
              >
                {isLaunching ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Zap className="h-4 w-4" />
                )}
              </Button>
            )}
            {onUninstall && (
              <Button
                variant="ghost"
                size="icon"
                title={t("tools.uninstall", "卸载")}
                onClick={onUninstall}
                disabled={isUninstalling}
                className="h-8 w-8 text-muted-foreground hover:text-destructive data[state=disabled]:opacity-50"
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
                variant="secondary"
                size="sm"
                onClick={onUpdate}
                className="text-[12px]"
              >
                <RefreshCw className="h-3 w-3 mr-1" />
                {t("tools.update", "更新")}
              </Button>
            )}
          </>
        )}

        {variant === "available" && !isInstalling && (
          <Button size="sm" onClick={onInstall} className="text-[13px]">
            <Download className="h-3 w-3 mr-1.5" />
            {t("tools.install", "安装")}
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
