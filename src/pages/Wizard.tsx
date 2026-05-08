// 装机向导页 — 单页整合：网络检测 + 安装目录 + 工具选择

import { useState, useCallback } from 'react';
import { Loader2, CheckCircle, AlertTriangle, ExternalLink, FolderOpen, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { InstallProgress } from '@/components/tools/InstallProgress';
import { ToolIcon } from '@/components/tools/ToolIcon';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Checkbox } from '@/components/ui/checkbox';
import { Card } from '@/components/ui/card';
import { useCheckNetwork, useResolveInstallPlan, useExecuteInstallPlan } from '@/hooks/useTools';
import { useInstallProgress } from '@/hooks/useInstallProgress';
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

  const toggleAll = useCallback(() => {
    if (selectedTools.size === AVAILABLE_TOOLS.length) {
      setSelectedTools(new Set());
    } else {
      setSelectedTools(new Set(AVAILABLE_TOOLS.map((t) => t.id)));
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
      <div className="text-center mb-8">
        <h1 className="text-2xl font-bold">{t('tools.wizardTitle', '装机向导')}</h1>
      </div>

      <div className="space-y-8">
        {/* 1. 工具选择 */}
        <section className="rounded-lg border p-4 space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">
              {t('tools.wizardTools', '选择工具')}
            </span>
            <Button variant="link" size="sm" onClick={toggleAll} className="text-xs">
              {selectedTools.size === AVAILABLE_TOOLS.length
                ? t('tools.none', '全部取消')
                : t('tools.all', '全部勾选')}
            </Button>
          </div>

          <div className="space-y-1">
            {AVAILABLE_TOOLS.map((tool) => (
              <Card
                key={tool.id}
                className="flex items-center gap-3 p-3 cursor-pointer hover:border-blue-500/50 transition-colors"
                onClick={() => toggleTool(tool.id)}
              >
                <Checkbox
                  checked={selectedTools.has(tool.id)}
                  onCheckedChange={() => toggleTool(tool.id)}
                />
                <ToolIcon toolId={tool.id} size={20} className="flex-shrink-0" />
                <div className="flex-1 min-w-0">
                  <Label className="text-sm font-medium cursor-pointer">{tool.name}</Label>
                  <p className="text-xs text-muted-foreground">{tool.description}</p>
                </div>
              </Card>
            ))}
          </div>

          <p className="text-xs text-muted-foreground text-center pt-1">
            {t('tools.autoDepsNote', '所需依赖（如 Node.js、Git）将在安装过程中自动配置，无需手动处理')}
          </p>
        </section>

        {/* 2. 安装目录 */}
        <section className="rounded-lg border p-4 space-y-3">
          <Label htmlFor="install-root" className="text-sm font-medium">
            {t('tools.installRoot', '安装根目录')}
          </Label>
          <div className="flex gap-2">
            <Input
              id="install-root"
              value={rootPath}
              onChange={(e) => setRootPath(e.target.value)}
              placeholder="D:\AITools"
              className="font-mono text-sm"
            />
            <Button
              variant="outline"
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
          <p className="text-xs text-muted-foreground">
            {t('tools.installRootHint', '所有工具将安装到此目录下的子目录中')}
          </p>
        </section>

        {/* 3. 网络状态 */}
        <section className="rounded-lg border p-4">
          <div className="flex items-center gap-4">
            {(netLoading || netFetching) && <Loader2 className="h-4 w-4 animate-spin text-blue-500 flex-shrink-0" />}
            {!netLoading && !netFetching && netOk && <CheckCircle className="h-4 w-4 text-green-500 flex-shrink-0" />}
            {!netLoading && !netFetching && (netError || netStatus?.errorMessage) && <AlertTriangle className="h-4 w-4 text-amber-500 flex-shrink-0" />}

            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-3 text-xs">
                <span className="text-muted-foreground">连通性:</span>
                {(netLoading || netFetching) ? (
                  <span className="text-muted-foreground">检测中...</span>
                ) : (
                  <>
                    <span className={netStatus?.githubReachable ? 'text-green-600' : 'text-red-500'}>
                      GitHub {netStatus?.githubReachable ? '✓' : '✗'}
                    </span>
                    <span className={netStatus?.npmReachable ? 'text-green-600' : 'text-red-500'}>
                      npm {netStatus?.npmReachable ? '✓' : '✗'}
                    </span>
                    <span className={netStatus?.youtubeReachable ? 'text-green-600' : 'text-muted-foreground'}>
                      YouTube {netStatus?.youtubeReachable ? '✓' : '✗'}
                    </span>
                  </>
                )}
              </div>
              {netStatus?.errorMessage && (
                <p className="text-xs text-amber-600 dark:text-amber-400 mt-1">{netStatus.errorMessage}</p>
              )}
            </div>

            <Button variant="ghost" size="sm" onClick={() => retryNet()} title="刷新">
              <RefreshCw className="h-3 w-3" />
            </Button>
          </div>

          {/* 网络问题解决指引 */}
          {!netLoading && !netFetching && !netOk && (
            <div className="mt-3 pt-3 border-t border-border text-xs text-muted-foreground space-y-2">
              <p>
                {netStatus?.youtubeReachable
                  ? '国际网络正常，但 GitHub/npm 可能被屏蔽。'
                  : '国际网络未连通。'}
              </p>
              <Button variant="outline" size="sm" asChild>
                <a href="https://github.com/unbound9527/agenticboot/blob/main/docs/network-troubleshooting.md" target="_blank" rel="noopener noreferrer">
                  <ExternalLink className="h-3 w-3 mr-1" />
                  查看网络问题解决指南
                </a>
              </Button>
            </div>
          )}
        </section>

        {/* 操作按钮 */}
        <div className="flex justify-center gap-4 pt-2">
          <Button
            variant="outline"
            size="lg"
            onClick={onComplete}
          >
            {t('tools.skipForNow', '跳过')}
          </Button>
          <Button
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
    </div>
  );
}
