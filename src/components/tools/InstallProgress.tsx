// 安装进度展示组件

import { CheckCircle, Loader2, XCircle, Dot } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { useInstallProgress } from '@/hooks/useInstallProgress';
import { useTranslation } from 'react-i18next';
import type { InstallPlan } from '@/types/tools';

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
        getToolProgress(s.toolId)?.phase === 'complete' ||
        getToolProgress(s.toolId)?.phase === 'skipped'
    ).length /
      installPlan.steps.length) *
      100
  );

  const getPhaseLabel = (phase: string | undefined): string => {
    switch (phase) {
      case 'downloading':
        return t('tools.phaseDownloading', '下载中...');
      case 'extracting':
        return t('tools.phaseExtracting', '解压中...');
      case 'installing':
        return t('tools.phaseInstalling', '安装中...');
      case 'configuring':
        return t('tools.phaseConfiguring', '配置中...');
      case 'complete':
        return t('tools.phaseComplete', '完成');
      case 'error':
        return t('tools.phaseError', '失败');
      case 'skipped':
        return t('tools.phaseSkipped', '已安装，跳过');
      default:
        return t('tools.phasePending', '等待中...');
    }
  };

  return (
    <div className="flex flex-col space-y-6 py-8">
      <h2 className="text-xl font-semibold text-center">
        {t('tools.installing', '安装中...')}
      </h2>

      {/* 总进度条 */}
      <div className="space-y-1">
        <div className="flex justify-between text-sm">
          <span>{overallPercent}%</span>
          <span className="text-muted-foreground">
            {Math.round(
              (overallPercent * installPlan.steps.length) / 100
            )}{' '}
            / {installPlan.steps.length}
          </span>
        </div>
        <Progress value={overallPercent} className="h-2" />
      </div>

      {/* 步骤列表 */}
      <div className="space-y-2">
        {installPlan.steps.map((step) => {
          const progress = getToolProgress(step.toolId);
          const isComplete =
            step.isInstalled || progress?.phase === 'complete' || progress?.phase === 'skipped';
          const isError = progress?.phase === 'error';
          const isActive =
            progress &&
            !['complete', 'error', 'skipped'].includes(progress.phase);

          return (
            <div
              key={step.toolId}
              className="flex items-center gap-3 rounded-lg border px-4 py-3"
            >
              {/* 状态图标 */}
              {isComplete && (
                <CheckCircle className="h-5 w-5 flex-shrink-0 text-green-500" />
              )}
              {isError && (
                <XCircle className="h-5 w-5 flex-shrink-0 text-red-500" />
              )}
              {isActive && (
                <Loader2 className="h-5 w-5 flex-shrink-0 animate-spin text-blue-500" />
              )}
              {!progress && (
                <Dot className="h-5 w-5 flex-shrink-0 text-muted-foreground" />
              )}

              {/* 工具信息 */}
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium truncate">
                    {step.toolName}
                  </span>
                  <Badge variant="secondary" className="text-xs">
                    {step.reason === 'selected'
                      ? t('tools.badgeSelected', '已选择')
                      : t('tools.badgeDependency', '依赖')}
                  </Badge>
                  {step.isInstalled && !progress && (
                    <Badge
                      variant="outline"
                      className="text-xs text-muted-foreground"
                    >
                      {t('tools.installed', '已安装')}
                    </Badge>
                  )}
                </div>
                <p className="text-xs text-muted-foreground">
                  {getPhaseLabel(progress?.phase)}
                  {isError && progress?.message && (
                    <span className="text-red-500 ml-2">
                      - {progress.message}
                    </span>
                  )}
                </p>

                {/* 单步进度条 */}
                {isActive && (
                  <Progress
                    value={progress?.percent ?? 0}
                    className="h-1 mt-2"
                  />
                )}
              </div>
            </div>
          );
        })}
      </div>

      {/* 完成和错误处理 */}
      {allComplete && !hasErrors && (
        <div className="text-center space-y-4 pt-4">
          <CheckCircle className="h-16 w-16 text-green-500 mx-auto" />
          <p className="text-lg font-medium">
            {t('tools.installComplete', '安装完成！')}
          </p>
          <Button onClick={onComplete}>
            {t('tools.enterManager', '进入管理')}
          </Button>
        </div>
      )}

      {hasErrors && (
        <div className="text-center space-y-3 pt-4">
          <XCircle className="h-16 w-16 text-red-500 mx-auto" />
          <p className="text-sm text-muted-foreground">
            {t('tools.installPartial', '部分工具安装失败，可稍后重试')}
          </p>
          <div className="flex justify-center gap-3">
            <Button variant="outline" onClick={onComplete}>
              {t('tools.skipForNow', '暂时跳过')}
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
