// 装机向导页 — 单页整合：网络检测 + 安装目录 + 工具选择

import { useState, useCallback, useEffect } from 'react';
import { Loader2, CheckCircle, AlertTriangle, ExternalLink, FolderOpen, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { InstallProgress } from '@/components/tools/InstallProgress';
import { NetworkHelpDialog } from '@/components/tools/NetworkHelpDialog';
import { ToolIcon } from '@/components/tools/ToolIcon';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Checkbox } from '@/components/ui/checkbox';
import { Card } from '@/components/ui/card';
import { useCheckNetwork, useResolveInstallPlan, useExecuteInstallPlan } from '@/hooks/useTools';
import { useInstallProgress } from '@/hooks/useInstallProgress';
import { toolsApi } from '@/lib/api/tools';
import type { InstallPlan } from '@/types/tools';

const AVAILABLE_TOOLS: { id: string; name: string; description: string }[] = [
  { id: 'claude-code-cli', name: 'Claude Code (CLI)', description: 'Anthropic 官方 CLI AI 编程助手' },
  { id: 'claude-code-desktop', name: 'Claude Code (桌面版)', description: 'Claude Code 桌面应用' },
  { id: 'codex-cli', name: 'Codex (CLI)', description: 'OpenAI 官方 CLI 编程助手' },
  { id: 'codex-desktop', name: 'Codex (桌面版)', description: 'Codex 桌面应用' },
  { id: 'gemini-cli', name: 'Gemini CLI', description: 'Google Gemini CLI 编程助手' },
  { id: 'opencode-cli', name: 'OpenCode (CLI)', description: '开源 AI 编程工具' },
  { id: 'opencode-desktop', name: 'OpenCode (桌面版)', description: 'OpenCode 桌面应用' },
  { id: 'openclaw', name: 'OpenClaw', description: '可编程 AI 编码引擎' },
  { id: 'hermes', name: 'Hermes (Web UI)', description: '多供应商 AI 编程助手，Web UI 交互' },
];

const DEFAULT_ROOT = 'D:\\AITools';

interface WizardProps {
  onComplete: () => void;
}

export function Wizard({ onComplete }: WizardProps) {
  const { t } = useTranslation();
  const [rootPath, setRootPath] = useState(DEFAULT_ROOT);
  const [selectedTools, setSelectedTools] = useState<Set<string>>(
    new Set(AVAILABLE_TOOLS.map((t) => t.id))
  );
  const [installPlan, setInstallPlan] = useState<InstallPlan | null>(null);
  const [started, setStarted] = useState(false);
  const [helpOpen, setHelpOpen] = useState(false);
  const [installedIds, setInstalledIds] = useState<Set<string>>(new Set());

  // 一次性检测本地已安装工具
  useEffect(() => {
    const ids = AVAILABLE_TOOLS.map((t) => t.id);
    toolsApi.detectTools(ids, rootPath).then((results) => {
      const detected = new Set<string>();
      results.forEach((r, i) => {
        if (r.installed) detected.add(ids[i]);
      });
      setInstalledIds(detected);
      // 已安装的工具从勾选列表中移除
      setSelectedTools((prev) => {
        const next = new Set(prev);
        detected.forEach((id) => next.delete(id));
        return next;
      });
    }).catch(() => {});
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const { data: netStatus, isLoading: netLoading, isFetching: netFetching, isError: netError, refetch: retryNet } = useCheckNetwork();
  const resolvePlan = useResolveInstallPlan();
  const executePlan = useExecuteInstallPlan();
  const { resetProgress } = useInstallProgress();

  const netOk = !netLoading && !netFetching && !netError && !netStatus?.errorMessage;

  const handleStartInstall = useCallback(() => {
    const toolIds = [...selectedTools];
    if (toolIds.length === 0) {
      toast.error(t('tools.noToolsSelected', '请至少选择一个工具'));
      return;
    }

    resolvePlan.mutate({ toolIds, installRoot: rootPath || undefined }, {
      onSuccess: (plan) => {
        setInstallPlan(plan);
        resetProgress();
        setStarted(true);
        executePlan.mutate(
          { plan, rootPath },
          {
            onError: (err) => {
              toast.error(
                t('tools.installFailed', '安装失败: {{error}}', { error: String(err) })
              );
            },
          }
        );
      },
      onError: (err) => {
        toast.error(
          t('tools.resolvePlanFailed', '解析安装计划失败: {{error}}', { error: String(err) })
        );
      },
    });
  }, [selectedTools, rootPath, resolvePlan, executePlan, resetProgress, t]);

  const toggleTool = useCallback((id: string) => {
    setSelectedTools((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  // 未安装的工具 ID 列表
  const availableIds = AVAILABLE_TOOLS.map((t) => t.id).filter((id) => !installedIds.has(id));

  const toggleAll = useCallback(() => {
    if (selectedTools.size === availableIds.length) {
      setSelectedTools(new Set());
    } else {
      setSelectedTools(new Set(availableIds));
    }
  }, [selectedTools.size]);

  // 安装进行中 → 显示进度
  if (started && installPlan) {
    return (
      <div className="px-6 py-6">
        <div className="text-center mb-8">
          <h1 className="text-2xl font-bold">{t('tools.wizardInstall', '安装中')}</h1>
        </div>
        <InstallProgress installPlan={installPlan} onComplete={onComplete} />
      </div>
    );
  }

  return (
    <div className="px-6 py-6">
      {/* 标题栏 - 粗野主义窗口风格 */}
      <div className="flex items-center gap-3 mb-8">
        <div className="brutal-window-bar border-2 border-[#111] shadow-[4px_4px_0_#111]">
          <div className="brutal-window-title">
            ~/projects/AgenticBoot
          </div>
        </div>
        <h1 className="text-3xl font-black tracking-tight">
          <span className="brutal-highlight">{t('tools.wizardTitle', '装机向导')}</span>
        </h1>
      </div>

      <div className="space-y-8">
        {/* 1. 工具选择 - 粗野主义卡片 */}
        <section className="border-[3px] border-[#111] bg-white p-4 space-y-3 shadow-[6px_6px_0_#111]">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="text-sm font-black uppercase tracking-wide">
                {t('tools.wizardTools', '选择工具')}
              </span>
              <span className="brutal-tag text-xs"># 告别折腾</span>
              <span className="brutal-tag brutal-tag-primary text-xs">拯救C盘</span>
            </div>
            <Button variant="link" size="sm" onClick={toggleAll} className="text-xs font-bold text-[#111] hover:text-[#FF5A36]">
              {selectedTools.size === availableIds.length
                ? t('tools.none', '全部取消')
                : t('tools.all', '全选')}
            </Button>
          </div>

          <div className="space-y-2">
            {AVAILABLE_TOOLS.map((tool) => {
              const isInstalled = installedIds.has(tool.id);
              return (
                <Card
                  key={tool.id}
                  variant="brutal"
                  className={`flex items-center gap-3 p-3 transition-all cursor-pointer ${
                    isInstalled
                      ? 'opacity-60'
                      : 'hover:shadow-[4px_4px_0_#111] hover:-translate-x-[2px] hover:-translate-y-[2px]'
                  }`}
                  onClick={() => !isInstalled && toggleTool(tool.id)}
                >
                  {isInstalled ? (
                    <CheckCircle className="h-5 w-5 text-green-500 flex-shrink-0" />
                  ) : (
                    <div className={`brutal-checkbox flex-shrink-0 ${selectedTools.has(tool.id) ? 'brutal-checkbox-checked' : ''}`}>
                      {selectedTools.has(tool.id) && '✓'}
                    </div>
                  )}
                  <ToolIcon toolId={tool.id} size={20} className="flex-shrink-0" />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <Label className={`text-sm font-bold ${isInstalled ? '' : 'cursor-pointer'}`}>
                        {tool.name}
                      </Label>
                      {isInstalled && (
                        <Badge variant="outline" className="text-xs font-bold border-2 border-[#111] text-green-600">
                          已安装
                        </Badge>
                      )}
                    </div>
                    <p className="text-xs text-[#666]">{tool.description}</p>
                  </div>
                </Card>
              );
            })}
          </div>

          <p className="text-xs text-center pt-2 font-medium text-[#666] border-t-2 border-dashed border-[#ccc] mt-3">
            {t('tools.autoDepsNote', '所需依赖（如 Node.js、Git）将在安装过程中自动配置')}
          </p>
        </section>

        {/* 2. 安装目录 */}
        <section className="border-[3px] border-[#111] bg-white p-4 space-y-3 shadow-[6px_6px_0_#111]">
          <Label htmlFor="install-root" className="text-sm font-black uppercase tracking-wide">
            {t('tools.installRoot', '安装目录')}
          </Label>
          <div className="flex gap-2">
            <Input
              id="install-root"
              value={rootPath}
              onChange={(e) => setRootPath(e.target.value)}
              placeholder="D:\AITools"
              className="font-mono text-sm border-2 border-[#111] focus:border-[#FF5A36] rounded-sm"
            />
            <Button
              variant="brutal-outline"
              size="icon"
              title={t('tools.browseFolder', '浏览文件夹')}
              onClick={() => {
                import('@tauri-apps/plugin-dialog').then(({ open }) => {
                  open({ directory: true, multiple: false }).then((result) => {
                    if (result && typeof result === 'string') setRootPath(result);
                  });
                });
              }}
            >
              <FolderOpen className="h-4 w-4" />
            </Button>
          </div>
          <p className="text-xs text-[#666]">
            {t('tools.installRootHint', '所有工具将安装到此目录下的子目录中')}
          </p>
        </section>

        {/* 3. 网络状态 */}
        <section className="border-[3px] border-[#111] bg-white p-4 shadow-[6px_6px_0_#111]">
          <div className="flex items-center gap-4">
            {(netLoading || netFetching) && <Loader2 className="h-4 w-4 animate-spin text-[#FF5A36] flex-shrink-0" />}
            {!netLoading && !netFetching && netOk && <CheckCircle className="h-4 w-4 text-green-500 flex-shrink-0" />}
            {!netLoading && !netFetching && (netError || netStatus?.errorMessage) && <AlertTriangle className="h-4 w-4 text-amber-500 flex-shrink-0" />}

            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-3 text-xs font-mono">
                <span className="font-bold text-[#111]">连通性:</span>
                {(netLoading || netFetching) ? (
                  <span className="text-[#666]">检测中...</span>
                ) : (
                  <>
                    <span className={`font-bold ${netStatus?.githubReachable ? 'text-green-500' : 'text-red-500'}`}>
                      GitHub {netStatus?.githubReachable ? '✓' : '✗'}
                    </span>
                    <span className={`font-bold ${netStatus?.npmReachable ? 'text-green-500' : 'text-red-500'}`}>
                      npm {netStatus?.npmReachable ? '✓' : '✗'}
                    </span>
                    <span className={`font-bold ${netStatus?.youtubeReachable ? 'text-green-500' : 'text-[#999]'}`}>
                      YouTube {netStatus?.youtubeReachable ? '✓' : '✗'}
                    </span>
                  </>
                )}
              </div>
              {netStatus?.errorMessage && (
                <p className="text-xs text-amber-600 mt-1 font-bold">{netStatus.errorMessage}</p>
              )}
            </div>

            <Button variant="ghost" size="sm" onClick={() => retryNet()} title="刷新" className="text-[#111]">
              <RefreshCw className="h-3 w-3" />
            </Button>
          </div>

          {/* 网络不通时提供解决链接 */}
          {!netLoading && !netFetching && !netOk && (
            <button
              onClick={() => setHelpOpen(true)}
              className="inline-flex items-center gap-1 mt-3 text-xs font-bold text-[#FF5A36] hover:underline"
            >
              <ExternalLink className="h-3 w-3" />
              网络不通？点击查看解决方法
            </button>
          )}
        </section>

        {/* 操作按钮 */}
        <div className="flex justify-center gap-4 pt-2">
          <Button
            variant="brutal-outline"
            size="lg"
            onClick={onComplete}
          >
            {t('tools.skipForNow', '跳过')}
          </Button>
          <Button
            variant="brutal"
            size="lg"
            onClick={handleStartInstall}
            disabled={resolvePlan.isPending || !rootPath.trim()}
            className="px-12"
          >
            {resolvePlan.isPending
              ? t('tools.resolving', '解析中...')
              : t('tools.startInstall', '开始安装')}
          </Button>
        </div>
      </div>

      <NetworkHelpDialog open={helpOpen} onOpenChange={setHelpOpen} />
    </div>
  );
}
