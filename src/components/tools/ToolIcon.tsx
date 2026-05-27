// 工具图标映射组件

import { Terminal, Monitor, Wrench } from 'lucide-react';
import ClaudeSvg from '@/icons/extracted/claude.svg?url';
import OpenAISvg from '@/icons/extracted/openai.svg?url';
import GeminiSvg from '@/icons/extracted/gemini.svg?url';
import OpenClawSvg from '@/icons/extracted/claw.svg?url';
import OpenCodeSvg from '@/icons/extracted/opencode-logo-light.svg?url';
import HermesPng from '@/icons/extracted/hermes.png';

interface ToolIconProps {
  toolId: string;
  size?: number;
  className?: string;
  spinning?: boolean;
}

export function ToolIcon({ toolId, size = 20, className = '', spinning = false }: ToolIconProps) {
  const spinningClass = spinning ? 'animate-spin' : '';
  const cls = `flex-shrink-0 rounded-md flex items-center justify-center ${className} ${spinningClass}`;

  switch (toolId) {
    case 'claude-code-cli':
      return <img src={ClaudeSvg} width={size} height={size} className={cls} alt="Claude" />;
    case 'claude-code-desktop':
      return (
        <div className={`${cls} bg-[#F07439]/10 relative`} style={{ width: size + 8, height: size + 8 }}>
          <img src={ClaudeSvg} width={size - 2} height={size - 2} alt="Claude" />
          <Monitor className="absolute -bottom-0.5 -right-0.5 w-3 h-3 text-muted-foreground" />
        </div>
      );
    case 'codex-cli':
      return <img src={OpenAISvg} width={size} height={size} className={`${cls} dark:brightness-0 dark:invert`} alt="Codex" />;
    case 'codex-desktop':
      return (
        <div className={`${cls} relative`} style={{ width: size + 8, height: size + 8 }}>
          <img src={OpenAISvg} width={size - 2} height={size - 2} className="dark:brightness-0 dark:invert" alt="Codex" />
          <Monitor className="absolute -bottom-0.5 -right-0.5 w-3 h-3 text-muted-foreground" />
        </div>
      );
    case 'gemini-cli':
      return <img src={GeminiSvg} width={size} height={size} className={cls} alt="Gemini" />;
    case 'opencode-cli':
      return <img src={OpenCodeSvg} width={size} height={size} className={cls} alt="OpenCode" />;
    case 'opencode-desktop':
      return (
        <div className={`${cls} relative`} style={{ width: size + 8, height: size + 8 }}>
          <img src={OpenCodeSvg} width={size - 2} height={size - 2} alt="OpenCode" />
          <Monitor className="absolute -bottom-0.5 -right-0.5 w-3 h-3 text-muted-foreground" />
        </div>
      );
    case 'openclaw':
      return <img src={OpenClawSvg} width={size} height={size} className={cls} alt="OpenClaw" />;
    case 'hermes':
      return <img src={HermesPng} width={size} height={size} className={cls} alt="Hermes" />;
    case 'nodejs':
      return (
        <div className={`${cls} bg-green-500/10`} style={{ width: size + 8, height: size + 8 }}>
          <Terminal className="w-4 h-4 text-green-500" />
        </div>
      );
    case 'git':
      return (
        <div className={`${cls} bg-orange-500/10`} style={{ width: size + 8, height: size + 8 }}>
          <Terminal className="w-4 h-4 text-orange-500" />
        </div>
      );
    default:
      return (
        <div className={`${cls} bg-muted`} style={{ width: size + 8, height: size + 8 }}>
          <Wrench className="w-4 h-4 text-muted-foreground" />
        </div>
      );
  }
}
