// 网络问题解决指南弹窗

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useTranslation } from 'react-i18next';

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function NetworkHelpDialog({ open, onOpenChange }: Props) {
  const { t } = useTranslation();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{t('tools.networkHelpTitle', '网络问题解决指南')}</DialogTitle>
        </DialogHeader>

        <div className="text-sm text-muted-foreground space-y-4">
          <div>
            <h3 className="text-foreground font-medium mb-2">1. 使用代理 / VPN（推荐）</h3>
            <p className="mb-2">优先使用代理或 VPN，一劳永逸解决所有网络问题。</p>
            <div className="space-y-1 text-xs">
              <p className="font-medium text-foreground/80">Windows 客户端：</p>
              <ul className="list-disc pl-5 space-y-0.5">
                <li>v2rayN — github.com/2dust/v2rayN</li>
                <li>clash-verge-rev — github.com/clash-verge-rev/clash-verge-rev</li>
                <li>nekoray — github.com/MatsuriDayo/nekoray</li>
              </ul>
              <p className="font-medium text-foreground/80 mt-2">多平台 / 内核：</p>
              <ul className="list-disc pl-5 space-y-0.5">
                <li>Hiddify — github.com/hiddify/hiddify-next</li>
                <li>sing-box — github.com/SagerNet/sing-box</li>
                <li>mihomo (Clash Meta) — github.com/MetaCubeX/mihomo</li>
              </ul>
            </div>
          </div>

          <div>
            <h3 className="text-foreground font-medium mb-2">2. 配置 npm 国内镜像</h3>
            <code className="block text-xs bg-muted px-2 py-1 rounded select-all">
              npm config set registry https://registry.npmmirror.com
            </code>
          </div>

          <div>
            <h3 className="text-foreground font-medium mb-2">3. 命令行代理</h3>
            <code className="block text-xs bg-muted px-2 py-1 rounded select-all mb-1">
              set HTTP_PROXY=http://127.0.0.1:端口
            </code>
            <code className="block text-xs bg-muted px-2 py-1 rounded select-all">
              set HTTPS_PROXY=http://127.0.0.1:端口
            </code>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
