import { Trash2, Download, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";
import { ToolIcon } from "@/components/tools/ToolIcon";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import type { InstallProgress, InstalledTool, ToolMeta } from "@/types/tools";

type ToolData = InstalledTool | ToolMeta;

interface ToolCardProps {
  tool: ToolData;
  variant: "installed" | "available";
  onInstall?: () => void;
  onUninstall?: () => void;
  onUpdate?: () => void;
  progress?: InstallProgress | null;
}

export function ToolCard({
  tool,
  variant,
  onInstall,
  onUninstall,
  onUpdate,
  progress,
}: ToolCardProps) {
  const { t } = useTranslation();

  const isInstalling =
    progress && !["complete", "error", "skipped"].includes(progress.phase);

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
          <Progress value={progress.percent} className="h-1 mt-2" />
        )}
      </div>

      <div className="flex-shrink-0 flex items-center gap-2">
        {variant === "installed" && (
          <>
            <Badge
              variant="secondary"
              className="text-[11px] px-2 py-0.5 bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-400"
            >
              {t("tools.installed", "已安装")}
              {"version" in tool && tool.version ? ` v${tool.version}` : ""}
            </Badge>
            {onUninstall && (
              <Button
                variant="ghost"
                size="icon"
                title={t("tools.uninstall", "卸载")}
                onClick={onUninstall}
                className="h-8 w-8 text-muted-foreground hover:text-destructive"
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            )}
            {onUpdate && (
              <Button variant="secondary" size="sm" onClick={onUpdate} className="text-[12px]">
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
