// 安装进度展示组件

import { CheckCircle, Loader2, XCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
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
        return '> 正在下载...';
      case 'extracting':
        return '> 正在解压...';
      case 'installing':
        return '> 正在安装...';
      case 'configuring':
        return '> 正在配置...';
      case 'complete':
        return '> [OK] 安装完成';
      case 'error':
        return '> [ERROR] 安装失败';
      case 'skipped':
        return '> [SKIP] 已安装，跳过';
      default:
        return '> 等待中...';
    }
  };

  return (
    <div className="flex flex-col space-y-6 py-8">
      {/* 粗野主义终端卡片 */}
      <div className="brutal-terminal-card rounded-lg overflow-hidden">
        {/* 终端头部 */}
        <div className="bg-[#f4f4f4] border-b-4 border-[#111] px-4 py-3 flex justify-between items-center font-mono">
          <span className="text-sm font-bold text-[#111]">AgenticBoot UI v1.0</span>
          <span className="text-sm font-bold text-[#111]">_ □ ×</span>
        </div>

        {/* 终端内容 */}
        <div className="p-6 font-mono text-sm text-[#111] space-y-4">
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
                <div key={step.toolId} className="flex items-center gap-3">
                  {/* 状态图标 */}
                  {isComplete && (
                    <CheckCircle className="h-4 w-4 flex-shrink-0 text-green-500" />
                  )}
                  {isError && (
                    <XCircle className="h-4 w-4 flex-shrink-0 text-red-500" />
                  )}
                  {isActive && (
                    <Loader2 className="h-4 w-4 flex-shrink-0 animate-spin text-[#FF5A36]" />
                  )}

                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className={`font-bold ${isError ? 'line-through text-red-500' : ''}`}>
                        {step.toolName}
                      </span>
                      {isError && progress?.message && (
                        <span className="text-red-500 text-xs">
                          - {progress.message}
                        </span>
                      )}
                    </div>
                    <p className={`text-xs ${isActive ? 'text-[#666]' : isError ? 'text-red-400' : 'text-green-600'}`}>
                      {getPhaseLabel(progress?.phase)}
                    </p>
                    {/* 单步进度条 */}
                    {isActive && (
                      <Progress
                        value={progress?.percent ?? 0}
                        className="h-1 mt-1 bg-[#eee] [&>div]:bg-[#FF5A36]"
                      />
                    )}
                  </div>
                </div>
              );
            })}
          </div>

          {/* 总进度 */}
          <div className="border-t-2 border-dashed border-[#ccc] pt-4 space-y-1">
            <div className="flex justify-between text-xs font-bold">
              <span>总进度:</span>
              <span>{overallPercent}%</span>
            </div>
            <Progress value={overallPercent} className="h-3 bg-[#eee] [&>div]:bg-[#FF5A36]" />
            <p className="text-xs text-[#666]">
              {Math.round(
                (overallPercent * installPlan.steps.length) / 100
              )} / {installPlan.steps.length} 个工具
            </p>
          </div>
        </div>
      </div>

      {/* 完成和错误处理 */}
      {allComplete && !hasErrors && (
        <div className="text-center space-y-4 pt-4">
          <div className="inline-flex items-center justify-center w-20 h-20 bg-green-500 text-white border-4 border-[#111] shadow-[6px_6px_0_#111]">
            <CheckCircle className="h-10 w-10" />
          </div>
          <p className="text-xl font-black text-[#111]">
            {t('tools.installComplete', '安装完成！')}
          </p>
          <Button variant="brutal" onClick={onComplete}>
            {t('tools.enterManager', '进入管理')}
          </Button>
        </div>
      )}

      {hasErrors && (
        <div className="text-center space-y-3 pt-4">
          <div className="inline-flex items-center justify-center w-20 h-20 bg-red-500 text-white border-4 border-[#111] shadow-[6px_6px_0_#111]">
            <XCircle className="h-10 w-10" />
          </div>
          <p className="text-sm font-bold text-[#666]">
            {t('tools.installPartial', '部分工具安装失败，可稍后重试')}
          </p>
          <div className="flex justify-center gap-3">
            <Button variant="brutal-outline" onClick={onComplete}>
              {t('tools.skipForNow', '暂时跳过')}
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
