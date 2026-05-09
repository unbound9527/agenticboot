// 安装根目录配置组件

import { useState } from 'react';
import { FolderOpen, FolderTree } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useTranslation } from 'react-i18next';

interface PathConfigProps {
  onNext: (rootPath: string) => void;
  onBack: () => void;
}

const DEFAULT_ROOT = 'D:\\AgenticBoot';

const TOOL_PREVIEWS = [
  'claude-code-cli',
  'codex-cli',
  'gemini-cli',
  'opencode-cli',
  'openclaw',
  'hermes',
];

export function PathConfig({ onNext, onBack }: PathConfigProps) {
  const { t } = useTranslation();
  const [rootPath, setRootPath] = useState(DEFAULT_ROOT);

  return (
    <div className="flex flex-col space-y-6 py-8">
      <h2 className="text-xl font-semibold text-center">
        {t('tools.selectRoot', '选择安装目录')}
      </h2>

      {/* 路径输入 */}
      <div className="space-y-2">
        <Label htmlFor="install-root">
          {t('tools.installRoot', '安装根目录')}
        </Label>
        <div className="flex gap-2">
          <Input
            id="install-root"
            value={rootPath}
            onChange={(e) => setRootPath(e.target.value)}
            placeholder={DEFAULT_ROOT}
            className="font-mono text-sm"
          />
          <Button
            variant="outline"
            size="icon"
            title={t('tools.browseFolder', '浏览文件夹')}
            onClick={() => {
              // 使用 Tauri dialog 选择目录
              import('@tauri-apps/plugin-dialog').then(({ open }) => {
                open({ directory: true, multiple: false }).then((result) => {
                  if (result && typeof result === 'string') {
                    setRootPath(result);
                  }
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
      </div>

      {/* 目录预览 */}
      <div className="rounded-lg border bg-muted/50 p-4 space-y-2">
        <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
          <FolderTree className="h-4 w-4" />
          {t('tools.preview', '目录预览')}
        </div>
        <pre className="text-xs font-mono text-muted-foreground whitespace-pre">
          {rootPath}
          {'\\\n'}
          {'  bin\\          ← shim 脚本目录\n'}
          {TOOL_PREVIEWS.map((id) => `  ${id}\\\n`).join('')}
          {'  ...'}
        </pre>
      </div>

      {/* 操作按钮 */}
      <div className="flex justify-between pt-4">
        <Button variant="outline" onClick={onBack}>
          {t('tools.previous', '上一步')}
        </Button>
        <Button onClick={() => onNext(rootPath)} disabled={!rootPath.trim()}>
          {t('tools.next', '下一步')}
        </Button>
      </div>
    </div>
  );
}
