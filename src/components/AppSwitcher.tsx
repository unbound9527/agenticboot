import type { AppId } from "@/lib/api";
import type { VisibleApps } from "@/types";
import { ProviderIcon } from "@/components/ProviderIcon";
import { cn } from "@/lib/utils";

interface AppSwitcherProps {
  activeApp: AppId;
  onSwitch: (app: AppId) => void;
  visibleApps?: VisibleApps;
  compact?: boolean;
}

const ALL_APPS: AppId[] = [
  "claude",
  "claude-desktop",
  "codex",
  "gemini",
  "opencode",
  "openclaw",
  "hermes",
];
const STORAGE_KEY = "agenticboot-last-app";

export function AppSwitcher({
  activeApp,
  onSwitch,
  visibleApps,
  compact,
}: AppSwitcherProps) {
  const handleSwitch = (app: AppId) => {
    if (app === activeApp) return;
    localStorage.setItem(STORAGE_KEY, app);
    onSwitch(app);
  };
  const iconSize = 20;
  const appIconName: Record<AppId, string> = {
    claude: "claude",
    "claude-desktop": "claude",
    codex: "openai",
    gemini: "gemini",
    opencode: "opencode",
    openclaw: "openclaw",
    hermes: "hermes",
  };
  const appDisplayName: Record<AppId, string> = {
    claude: "Claude",
    "claude-desktop": "Claude Desktop",
    codex: "Codex",
    gemini: "Gemini",
    opencode: "OpenCode",
    openclaw: "OpenClaw",
    hermes: "Hermes",
  };

  // Filter apps based on visibility settings (default all visible)
  const appsToShow = ALL_APPS.filter((app) => {
    if (!visibleApps) return true;
    return visibleApps[app];
  });

  return (
    <div className="inline-flex gap-1 p-1 rounded-lg bg-muted/60">
      {appsToShow.map((app) => (
        <button
          key={app}
          type="button"
          onClick={() => handleSwitch(app)}
          className={cn(
            "group inline-flex items-center px-3 h-7 text-[13px] font-medium transition-all duration-150 rounded-md",
            activeApp === app
              ? "bg-background text-foreground shadow-sm"
              : "text-muted-foreground hover:text-foreground hover:bg-muted",
            compact ? "w-8 px-0 justify-center" : ""
          )}
        >
          <ProviderIcon
            icon={appIconName[app]}
            name={appDisplayName[app]}
            size={iconSize}
          />
          <span
            className={cn(
              "transition-all duration-200 whitespace-nowrap overflow-hidden",
              compact
                ? "max-w-0 opacity-0 ml-0"
                : "max-w-[120px] opacity-100 ml-2",
            )}
          >
            {appDisplayName[app]}
          </span>
        </button>
      ))}
    </div>
  );
}
