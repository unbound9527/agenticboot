// 工具管家页 — 日常管理已安装/未安装工具

import { useState, useCallback, useEffect } from 'react';
import { Settings, RefreshCw, FolderOpen } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { ToolCard } from '@/components/tools/ToolCard';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  useInstalledTools,
  useUninstallTool,
  useInstallRoot,
  useToolUpdates,
} from '@/hooks/useTools';
import { useInstallProgress } from '@/hooks/useInstallProgress';
import type { ToolMeta } from '@/types/tools';

// 所有已知工具的元信息（与 wizard 中的列表保持一致）
const ALL_TOOLS_META: ToolMeta[] = [
  { id: 'claude-code-cli', name: 'Claude Code (CLI)', description: 'Anthropic 官方 CLI AI 编程助手', icon: 'claude', category: 'ai-cli' },
  { id: 'claude-code-desktop', name: 'Claude Code (桌面版)', description: 'Claude Code 桌面应用', icon: 'claude', category: 'ai-cli' },
  { id: 'codex-cli', name: 'Codex (CLI)', description: 'OpenAI 官方 CLI 编程助手', icon: 'codex', category: 'ai-cli' },
  { id: 'codex-desktop', name: 'Codex (桌面版)', description: 'Codex 桌面应用', icon: 'codex', category: 'ai-cli' },
  { id: 'gemini-cli', name: 'Gemini CLI', description: 'Google Gemini CLI 编程助手', icon: 'gemini', category: 'ai-cli' },
  { id: 'opencode-cli', name: 'OpenCode (CLI)', description: '开源 AI 编程工具', icon: 'opencode', category: 'ai-cli' },
  { id: 'opencode-desktop', name: 'OpenCode (桌面版)', description: 'OpenCode 桌面应用', icon: 'opencode', category: 'ai-cli' },
  { id: 'openclaw', name: 'OpenClaw', description: '可编程 AI 编码引擎', icon: 'openclaw', category: 'ai-cli' },
  { id: 'hermes', name: 'Hermes (Web UI)', description: '多供应商 AI 编程助手，Web UI 交互', icon: 'hermes', category: 'ai-cli' },
];

interface ManagerProps {
  onInstallMore?: () => void;
}

export function Manager({ onInstallMore }: ManagerProps) {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState('installed');

  const { data: installedTools = [] } = useInstalledTools();
  const { data: installRoot } = useInstallRoot();
  const { data: updates = [] } = useToolUpdates();
  const uninstallTool = useUninstallTool();
  const { getToolProgress } = useInstallProgress();

  const [editRoot, setEditRoot] = useState(installRoot ?? '');
  useEffect(() => { setEditRoot(installRoot ?? ''); }, [installRoot]);

  const installedIds = new Set(installedTools.map((t) => t.id));
  const notInstalled = ALL_TOOLS_META.filter(
    (meta) => !installedIds.has(meta.id)
  );

  const handleUninstall = useCallback(
    (toolId: string) => {
      const root = installRoot ?? 'D:\\AITools';
      uninstallTool.mutate(
        { toolId, rootPath: root },
        {
          onSuccess: () => {
            toast.success(
              t('tools.uninstalled', '卸载成功')
            );
          },
          onError: (err) => {
            toast.error(
              t('tools.uninstallFailed', '卸载失败: {{error}}', {
                error: String(err),
              })
            );
          },
        }
      );
    },
    [installRoot, uninstallTool, t]
  );

  const handleSingleInstall = useCallback(
    (_toolId: string) => {
      // 单个工具安装 — 引导用户到向导页
      onInstallMore?.();
    },
    [onInstallMore]
  );

  return (
    <div className="px-6 py-6">
      {/* 标题栏 */}
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">
          {t('tools.manager', '软件管家')}
        </h1>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              if (updates.length > 0) {
                toast.info(
                  t('tools.updatesAvailable', '{{count}} 个工具可更新', {
                    count: updates.length,
                  })
                );
              } else {
                toast.success(
                  t('tools.allUpToDate', '所有工具均为最新版本')
                );
              }
            }}
          >
            <RefreshCw className="h-3 w-3 mr-1" />
            {t('tools.checkUpdates', '检查更新')}
          </Button>
        </div>
      </div>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="w-full">
          <TabsTrigger value="installed" className="flex-1">
            {t('tools.installedTab', '已安装')} ({installedTools.length})
          </TabsTrigger>
          <TabsTrigger value="available" className="flex-1">
            {t('tools.availableTab', '未安装')} ({notInstalled.length})
          </TabsTrigger>
        </TabsList>

        {/* 已安装 */}
        <TabsContent value="installed" className="space-y-2 mt-4">
          {installedTools.length === 0 && (
            <div className="text-center py-12 text-muted-foreground">
              <p className="text-lg">
                {t('tools.noToolsInstalled', '暂无已安装工具')}
              </p>
              <Button
                variant="link"
                onClick={onInstallMore}
                className="mt-2"
              >
                {t('tools.goInstall', '去安装')}
              </Button>
            </div>
          )}
          {installedTools.map((tool) => (
            <ToolCard
              key={tool.id}
              tool={tool}
              variant="installed"
              progress={getToolProgress(tool.id)}
              onUninstall={() => handleUninstall(tool.id)}
              onUpdate={
                updates.find((u) => u.toolId === tool.id)
                  ? () => handleSingleInstall(tool.id)
                  : undefined
              }
            />
          ))}
        </TabsContent>

        {/* 未安装 */}
        <TabsContent value="available" className="space-y-2 mt-4">
          {notInstalled.map((tool) => (
            <ToolCard
              key={tool.id}
              tool={tool}
              variant="available"
              onInstall={() => handleSingleInstall(tool.id)}
            />
          ))}
        </TabsContent>
      </Tabs>

      {/* 设置区域 */}
      <div className="mt-8 pt-4 border-t">
        <div className="flex items-center gap-3 text-sm text-muted-foreground">
          <Settings className="h-4 w-4 flex-shrink-0" />
          <span className="flex-shrink-0">{t('tools.installRoot', '安装根目录')}:</span>
          <input
            type="text"
            value={editRoot}
            onChange={(e) => setEditRoot(e.target.value)}
            placeholder="D:\AITools"
            className="flex-1 text-xs font-mono bg-muted px-2 py-1 rounded border border-border focus:outline-none focus:border-blue-500"
            onBlur={(e) => {
              const newRoot = e.target.value.trim();
              if (newRoot && newRoot !== installRoot) {
                import('@/lib/api/tools').then(({ toolsApi }) => {
                  toolsApi.setInstallRoot(newRoot).catch(() => {});
                });
              }
            }}
            onKeyDown={(e) => {
              if (e.key === 'Enter') (e.target as HTMLInputElement).blur();
            }}
          />
          <Button
            variant="ghost"
            size="icon"
            title={t('tools.browseFolder', '浏览文件夹')}
            onClick={() => {
              import('@tauri-apps/plugin-dialog').then(({ open }) => {
                open({ directory: true, multiple: false }).then((result) => {
                  if (result && typeof result === 'string') {
                    setEditRoot(result);
                    import('@/lib/api/tools').then(({ toolsApi }) => {
                      toolsApi.setInstallRoot(result).catch(() => {});
                    });
                  }
                });
              });
            }}
          >
            <FolderOpen className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  );
}
