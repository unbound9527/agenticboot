// 工具卡片组件 — 用于已安装/未安装工具展示

import { Trash2, Download, RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { ToolIcon } from '@/components/tools/ToolIcon';
import { useTranslation } from 'react-i18next';
import type { InstalledTool, ToolMeta, InstallProgress } from '@/types/tools';

type ToolData = InstalledTool | ToolMeta;

interface ToolCardProps {
  tool: ToolData;
  variant: 'installed' | 'available';
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
    progress &&
    !['complete', 'error', 'skipped'].includes(progress.phase);

  return (
    <Card
      variant="brutal"
      className="flex items-center gap-4 p-4 border-[3px] border-[#111] bg-white shadow-[4px_4px_0_#111]"
    >
      {/* 图标 */}
      <ToolIcon toolId={tool.id} size={22} />

      {/* 中间信息 */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-bold truncate">{tool.name}</span>
          {'category' in tool && (
            <Badge variant="secondary" className="text-xs font-bold border border-[#111]">
              {tool.category === 'dependency'
                ? t('tools.badgeDependency', '依赖')
                : t('tools.badgeTool', '工具')}
            </Badge>
          )}
        </div>
        <p className="text-xs text-[#666] truncate font-mono">
          {'description' in tool
            ? tool.description
            : tool.installPath || tool.name}
        </p>

        {/* 安装进度条 */}
        {isInstalling && progress && (
          <Progress value={progress.percent} className="h-2 mt-2 bg-[#eee] [&>div]:bg-[#FF5A36]" />
        )}
      </div>

      {/* 右侧操作 */}
      <div className="flex-shrink-0 flex items-center gap-2">
        {variant === 'installed' && (
          <>
            <Badge
              variant="outline"
              className="text-xs font-bold border-2 border-[#111] text-green-600 bg-green-50"
            >
              {t('tools.installed', '已安装')}
              {'version' in tool && tool.version
                ? ` v${tool.version}`
                : ''}
            </Badge>
            <Button
              variant="ghost"
              size="icon"
              title={t('tools.uninstall', '卸载')}
              onClick={onUninstall}
              className="hover:bg-red-100 hover:text-red-600"
            >
              <Trash2 className="h-4 w-4" />
            </Button>
            {onUpdate && (
              <Button
                variant="brutal-outline"
                size="sm"
                onClick={onUpdate}
              >
                <RefreshCw className="h-3 w-3 mr-1" />
                {t('tools.update', '更新')}
              </Button>
            )}
          </>
        )}

        {variant === 'available' && !isInstalling && (
          <Button variant="brutal" size="sm" onClick={onInstall}>
            <Download className="h-3 w-3 mr-1" />
            {t('tools.install', '安装')}
          </Button>
        )}

        {/* 安装进行中 */}
        {isInstalling && progress && (
          <span className="text-xs font-bold text-[#FF5A36]">
            {progress.percent}%
          </span>
        )}
      </div>
    </Card>
  );
}
