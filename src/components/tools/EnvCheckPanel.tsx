// 网络连接检测面板

import { Loader2, CheckCircle, AlertTriangle, ExternalLink } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useCheckNetwork } from '@/hooks/useTools';
import { useTranslation } from 'react-i18next';

interface EnvCheckPanelProps {
  onNext: () => void;
}

export function EnvCheckPanel({ onNext }: EnvCheckPanelProps) {
  const { t } = useTranslation();
  const { data, isLoading, isError, refetch } = useCheckNetwork();

  return (
    <div className="flex flex-col items-center justify-center py-12 space-y-6">
      {/* 状态图标 */}
      {isLoading && (
        <Loader2 className="h-16 w-16 animate-spin text-blue-500" />
      )}
      {!isLoading && data && !data.errorMessage && (
        <CheckCircle className="h-16 w-16 text-green-500" />
      )}
      {!isLoading && (isError || data?.errorMessage) && (
        <AlertTriangle className="h-16 w-16 text-amber-500" />
      )}

      {/* 状态文字 */}
      <div className="text-center space-y-2">
        <h2 className="text-xl font-semibold">
          {isLoading
            ? t('tools.checkNetwork', '检测网络连接...')
            : isError || data?.errorMessage
              ? t('tools.networkError', '网络连接异常')
              : t('tools.networkOk', '网络连接正常')}
        </h2>
        <p className="text-sm text-muted-foreground max-w-md">
          {isLoading
            ? t('tools.checkNetworkHint', '正在检测 GitHub 和 npm 源连通性...')
            : isError || data?.errorMessage
              ? t('tools.networkErrorHint', '请先解决网络问题再继续安装。')
              : t('tools.networkOkHint', '网络畅通，可以继续安装工具。')}
        </p>
        {data?.errorMessage && (
          <p className="text-xs text-amber-600 dark:text-amber-400 mt-1">
            {data.errorMessage}
          </p>
        )}
      </div>

      {/* 操作按钮 */}
      <div className="flex gap-3">
        {(isError || data?.errorMessage) && (
          <>
            <Button variant="outline" asChild>
              <a
                href="https://github.com/unbound9527/agenticboot/wiki/Network-Troubleshooting"
                target="_blank"
                rel="noopener noreferrer"
              >
                <ExternalLink className="h-4 w-4 mr-2" />
                {t('tools.networkGuide', '查看网络问题解决指南')}
              </a>
            </Button>
            <Button variant="outline" onClick={() => refetch()}>
              {t('tools.retry', '重试')}
            </Button>
          </>
        )}
        {!isLoading && !isError && !data?.errorMessage && (
          <Button onClick={onNext}>
            {t('tools.next', '下一步')}
          </Button>
        )}
      </div>
    </div>
  );
}
