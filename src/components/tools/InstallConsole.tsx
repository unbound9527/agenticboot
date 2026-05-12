import { useEffect, useState } from "react";
import { TerminalSquare, ChevronDown, ChevronUp, Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { ToolInstallSession } from "@/types/tools";

const CYCLING_CHARS = ["-", "\\", "|", "/"];

interface InstallConsoleProps {
  session: ToolInstallSession | null;
}

function formatTimestamp(timestamp: string) {
  try {
    return new Intl.DateTimeFormat(undefined, {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    }).format(new Date(timestamp));
  } catch {
    return timestamp;
  }
}

function CyclingChar({ isAnimating, className }: { isAnimating: boolean; className?: string }) {
  const [index, setIndex] = useState(0);

  useEffect(() => {
    if (!isAnimating) return;
    const id = setInterval(() => setIndex((i) => (i + 1) % CYCLING_CHARS.length), 150);
    return () => clearInterval(id);
  }, [isAnimating]);

  if (!isAnimating) return <span className={className}>-</span>;

  return <span className={className}>{CYCLING_CHARS[index]}</span>;
}

export function InstallConsole({ session }: InstallConsoleProps) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(true);
  const isActive = session?.status === "running";

  useEffect(() => {
    if (isActive) {
      setExpanded(true);
    }
  }, [isActive]);

  useEffect(() => {
    if (!session?.sessionId) {
      return;
    }

    setExpanded(true);
  }, [session?.sessionId]);

  if (!session) {
    return null;
  }

  return (
    <div className="overflow-hidden rounded-lg border border-border/60 bg-background shadow-sm">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex w-full items-center justify-between border-b border-border/60 bg-muted/50 px-4 py-2 transition-colors hover:bg-muted/70"
      >
        <div className="flex min-w-0 items-center gap-2">
          {isActive ? (
            <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground" />
          ) : (
            <TerminalSquare className="h-3.5 w-3.5 text-muted-foreground" />
          )}
          <span className="truncate text-[12px] font-medium">{session.toolName}</span>
          <Badge
            variant="secondary"
            className={
              session.status === "complete"
                ? "bg-emerald-100 px-1.5 py-0 text-[10px] text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-400"
                : session.status === "error"
                  ? "bg-red-100 px-1.5 py-0 text-[10px] text-red-700 dark:bg-red-900/40 dark:text-red-400"
                  : "px-1.5 py-0 text-[10px]"
            }
          >
            {session.status}
          </Badge>
        </div>
        {expanded ? (
          <ChevronUp className="h-4 w-4 text-muted-foreground" />
        ) : (
          <ChevronDown className="h-4 w-4 text-muted-foreground" />
        )}
      </button>

      {expanded && (
        <div>
          <div className="flex items-center justify-between border-b border-border/60 px-4 py-2">
            <span className="text-[10px] text-muted-foreground">
              {t("tools.installConsoleStarted", "Started")}: {formatTimestamp(session.startedAt)}
            </span>
          </div>
          <ScrollArea className="max-h-48">
            <div className="space-y-1 px-3 py-2 font-mono text-[11px] leading-4">
              {session.entries.length > 0 ? (
                session.entries.map((entry, index) => (
                  <div key={`${entry.timestamp}-${index}`} className="flex gap-2">
                    <span className="shrink-0 text-muted-foreground/60">
                      {formatTimestamp(entry.timestamp)}
                    </span>
                    <span
                      className={
                        entry.level === "error" || entry.level === "stderr"
                          ? "shrink-0 text-red-500"
                          : entry.kind === "output"
                            ? "shrink-0 text-muted-foreground"
                            : "shrink-0 text-foreground"
                      }
                    >
                      [{entry.kind}]
                    </span>
                    <span className="min-w-0 break-words">{entry.line}</span>
                  </div>
                ))
              ) : isActive ? (
                <div className="flex items-center gap-2 text-muted-foreground">
                  <CyclingChar isAnimating={true} className="text-muted-foreground" />
                  <span className="text-[11px]">
                    {t("tools.consoleWaiting", "Initializing...")}
                  </span>
                </div>
              ) : (
                <p className="text-[11px] text-muted-foreground">
                  {t("tools.installConsoleEmptyRaw", "No log output")}
                </p>
              )}
            </div>
          </ScrollArea>
        </div>
      )}
    </div>
  );
}
