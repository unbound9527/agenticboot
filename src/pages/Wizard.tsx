// 首次装机向导页

import { useState, useCallback } from 'react';
import { Check, ChevronRight } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { EnvCheckPanel } from '@/components/tools/EnvCheckPanel';
import { PathConfig } from '@/components/tools/PathConfig';
import { InstallProgress } from '@/components/tools/InstallProgress';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Card } from '@/components/ui/card';
import {
  useResolveInstallPlan,
  useExecuteInstallPlan,
} from '@/hooks/useTools';
import { useInstallProgress } from '@/hooks/useInstallProgress';
import type { InstallPlan } from '@/types/tools';

// 默认可选工具列表
const AVAILABLE_TOOLS: { id: string; name: string; description: string; depsNote?: string }[] = [
  { id: 'claude-code-cli', name: 'Claude Code (CLI)', description: 'Anthropic 官方 CLI AI 编程助手', depsNote: '需要 Node.js' },
  { id: 'claude-code-desktop', name: 'Claude Code (桌面版)', description: 'Claude Code 桌面应用，无需额外依赖' },
  { id: 'codex-cli', name: 'Codex (CLI)', description: 'OpenAI 官方 CLI 编程助手', depsNote: '需要 Node.js' },
  { id: 'codex-desktop', name: 'Codex (桌面版)', description: 'Codex 桌面应用，无需额外依赖' },
  { id: 'gemini-cli', name: 'Gemini CLI', description: 'Google Gemini CLI 编程助手', depsNote: '需要 Node.js' },
  { id: 'opencode-cli', name: 'OpenCode (CLI)', description: '开源 AI 编程工具', depsNote: '需要 Node.js' },
  { id: 'opencode-desktop', name: 'OpenCode (桌面版)', description: 'OpenCode 桌面应用，无需额外依赖' },
  { id: 'openclaw', name: 'OpenClaw', description: '可编程 AI 编码引擎', depsNote: '需要 Node.js' },
  { id: 'hermes', name: 'Hermes (Web UI)', description: '多供应商 AI 编程助手，Web UI 交互', depsNote: '需要 Node.js' },
];

const STEPS = ['network', 'path', 'tools', 'install'] as const;
type Step = (typeof STEPS)[number];

interface WizardProps {
  onComplete: () => void;
}

export function Wizard({ onComplete }: WizardProps) {
  const { t } = useTranslation();
  const [currentStep, setCurrentStep] = useState<Step>('network');
  const [rootPath, setRootPath] = useState('');
  const [selectedTools, setSelectedTools] = useState<Set<string>>(
    new Set(AVAILABLE_TOOLS.map((t) => t.id))
  );
  const [installPlan, setInstallPlan] = useState<InstallPlan | null>(null);

  const resolvePlan = useResolveInstallPlan();
  const executePlan = useExecuteInstallPlan();
  const { resetProgress } = useInstallProgress();

  const handleResolvePlan = useCallback(() => {
    const toolIds = [...selectedTools];
    if (toolIds.length === 0) {
      toast.error(t('tools.noToolsSelected', '请至少选择一个工具'));
      return;
    }

    resolvePlan.mutate(toolIds, {
      onSuccess: (plan) => {
        setInstallPlan(plan);
        resetProgress();
        setCurrentStep('install');
        // 开始执行安装
        executePlan.mutate(
          { plan, rootPath },
          {
            onError: (err) => {
              toast.error(
                t('tools.installFailed', '安装失败: {{error}}', {
                  error: String(err),
                })
              );
            },
          }
        );
      },
      onError: (err) => {
        toast.error(
          t('tools.resolvePlanFailed', '解析安装计划失败: {{error}}', {
            error: String(err),
          })
        );
      },
    });
  }, [selectedTools, rootPath, resolvePlan, executePlan, resetProgress, t]);

  const toggleTool = useCallback((id: string) => {
    setSelectedTools((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
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

  const stepIndex = STEPS.indexOf(currentStep);

  return (
    <div className="max-w-2xl mx-auto px-4 py-8">
      {/* 步骤指示器 */}
      <div className="flex items-center justify-center gap-2 mb-8">
        {STEPS.slice(0, -1).map((step, i) => (
          <div key={step} className="flex items-center gap-2">
            <div
              className={`flex items-center justify-center h-8 w-8 rounded-full text-sm font-medium transition-colors ${
                i <= stepIndex
                  ? 'bg-orange-500 text-white'
                  : 'bg-muted text-muted-foreground'
              }`}
            >
              {i < stepIndex ? (
                <Check className="h-4 w-4" />
              ) : (
                i + 1
              )}
            </div>
            {i < STEPS.length - 2 && (
              <ChevronRight className="h-4 w-4 text-muted-foreground" />
            )}
          </div>
        ))}
      </div>

      {/* 步骤标题 */}
      <div className="text-center mb-8">
        <h1 className="text-2xl font-bold">
          {currentStep === 'network' && t('tools.wizardNetwork', '环境检测')}
          {currentStep === 'path' && t('tools.wizardPath', '选择安装目录')}
          {currentStep === 'tools' && t('tools.wizardTools', '选择工具')}
          {currentStep === 'install' && t('tools.wizardInstall', '安装中')}
        </h1>
      </div>

      {/* 步骤内容 */}
      {currentStep === 'network' && (
        <EnvCheckPanel onNext={() => setCurrentStep('path')} />
      )}

      {currentStep === 'path' && (
        <PathConfig
          onNext={(path) => {
            setRootPath(path);
            setCurrentStep('tools');
          }}
          onBack={() => setCurrentStep('network')}
        />
      )}

      {currentStep === 'tools' && (
        <div className="flex flex-col space-y-6 py-8">
          <div className="flex justify-end">
            <Button
              variant="link"
              size="sm"
              onClick={toggleAll}
              className="text-xs"
            >
              {selectedTools.size === AVAILABLE_TOOLS.length
                ? t('tools.none', '全部取消')
                : t('tools.all', '全部勾选')}
            </Button>
          </div>

          <div className="space-y-2">
            {AVAILABLE_TOOLS.map((tool) => (
              <Card
                key={tool.id}
                className="flex items-center gap-4 p-4 cursor-pointer hover:border-orange-500/50 transition-colors"
                onClick={() => toggleTool(tool.id)}
              >
                <Checkbox
                  checked={selectedTools.has(tool.id)}
                  onCheckedChange={() => toggleTool(tool.id)}
                />
                <div className="flex-1 min-w-0">
                  <Label className="text-sm font-medium cursor-pointer">
                    {tool.name}
                  </Label>
                  <p className="text-xs text-muted-foreground">
                    {tool.description}
                    {tool.depsNote && (
                      <span className="ml-2 text-orange-500">
                        {tool.depsNote}
                      </span>
                    )}
                  </p>
                </div>
              </Card>
            ))}
          </div>

          <div className="flex justify-between pt-4">
            <Button
              variant="outline"
              onClick={() => setCurrentStep('path')}
            >
              {t('tools.previous', '上一步')}
            </Button>
            <Button
              onClick={handleResolvePlan}
              disabled={resolvePlan.isPending}
            >
              {resolvePlan.isPending
                ? t('tools.resolving', '解析中...')
                : t('tools.startInstall', '开始安装')}
            </Button>
          </div>
        </div>
      )}

      {currentStep === 'install' && installPlan && (
        <InstallProgress
          installPlan={installPlan}
          onComplete={onComplete}
        />
      )}
    </div>
  );
}
