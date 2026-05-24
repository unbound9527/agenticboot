import { useEffect, useMemo, useRef, useState } from "react";
import { TerminalSquare, ChevronDown, ChevronUp, Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { InstallProgress, ToolInstallSession } from "@/types/tools";

const HEARTBEAT_INTERVAL_MS = 2500;
const ENTRY_PAGE_SIZE = 40;
const HEARTBEAT_LINES = [
  "System: Creating install session...",
  "System: Verifying installer is still running...",
  "System: Waiting for next installer output...",
  "System: Applying the latest progress checkpoint...",
  "System: Finalizing this stage...",
];

interface InstallConsoleProps {
  session: ToolInstallSession | null;
  progress?: InstallProgress | null;
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

function getProgressSummary(progress: InstallProgress | null | undefined) {
  if (!progress) {
    return null;
  }

  return `System: ${progress.phase} ${progress.percent}% complete.`;
}

export function InstallConsole({ session, progress = null }: InstallConsoleProps) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(true);
  const [heartbeatTick, setHeartbeatTick] = useState(0);
  const [visibleEntryCount, setVisibleEntryCount] = useState(ENTRY_PAGE_SIZE);
  const isActive = session?.status === "running";
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const pendingScrollRestoreRef = useRef<{
    previousHeight: number;
    previousTop: number;
  } | null>(null);

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
    setHeartbeatTick(0);
    setVisibleEntryCount(ENTRY_PAGE_SIZE);
  }, [session?.sessionId]);

  useEffect(() => {
    if (!isActive) {
      return;
    }

    const id = window.setInterval(() => {
      setHeartbeatTick((current) => current + 1);
    }, HEARTBEAT_INTERVAL_MS);

    return () => {
      window.clearInterval(id);
    };
  }, [isActive]);

  const systemLines = [
    isActive ? HEARTBEAT_LINES[0] : null,
    getProgressSummary(progress),
    progress?.message ? `System: ${progress.message}` : null,
    ...HEARTBEAT_LINES.slice(1, heartbeatTick + 1),
  ].filter((line, index, lines): line is string => {
    return Boolean(line) && lines.indexOf(line) === index;
  });
  const entries = session?.entries ?? [];
  const hasMoreEntries = entries.length > visibleEntryCount;
  const visibleEntries = useMemo(() => {
    return hasMoreEntries
      ? entries.slice(-visibleEntryCount)
      : entries;
  }, [entries, hasMoreEntries, visibleEntryCount]);

  useEffect(() => {
    if (!session) {
      return;
    }

    const viewport = viewportRef.current;
    if (!viewport) {
      return;
    }

    if (pendingScrollRestoreRef.current) {
      const { previousHeight, previousTop } = pendingScrollRestoreRef.current;
      const nextHeight = viewport.scrollHeight;
      viewport.scrollTop = nextHeight - previousHeight + previousTop;
      pendingScrollRestoreRef.current = null;
      return;
    }

    viewport.scrollTop = viewport.scrollHeight;
  }, [entries.length, session, visibleEntryCount]);

  const loadOlderEntries = () => {
    if (!session) {
      return;
    }

    const viewport = viewportRef.current;
    if (hasMoreEntries && viewport) {
      pendingScrollRestoreRef.current = {
        previousHeight: viewport.scrollHeight,
        previousTop: viewport.scrollTop,
      };
    }

    setVisibleEntryCount((current) =>
      Math.min(entries.length, Math.max(current + ENTRY_PAGE_SIZE, entries.length)),
    );
  };

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
              <div
                ref={viewportRef}
                data-testid="install-console-viewport"
                className="max-h-44 overflow-y-auto"
                onScroll={(event) => {
                  if (event.currentTarget.scrollTop <= 8 && hasMoreEntries) {
                    loadOlderEntries();
                  }
                }}
              >
              {entries.length > 0 ? (
                <>
                  {hasMoreEntries && (
                    <div className="pb-2 text-center text-[10px] text-muted-foreground">
                      {t("tools.installConsoleLoadMore", "Scroll up to load earlier output")}
                    </div>
                  )}
                  {visibleEntries.map((entry, index) => (
                    <div key={`${entry.timestamp}-${index}`} className="flex gap-2">
                      <span className="shrink-0 text-muted-foreground/60">
                        {formatTimestamp(entry.timestamp)}
                      </span>
                      <span
                        className={
                          entry.source === "optimistic"
                            ? "shrink-0 text-blue-600/80"
                            : entry.level === "error" || entry.level === "stderr"
                            ? "shrink-0 text-red-500"
                            : entry.kind === "output"
                              ? "shrink-0 text-muted-foreground"
                              : "shrink-0 text-foreground"
                        }
                      >
                        [{entry.source === "optimistic" ? "system" : entry.kind}]
                      </span>
                      <span className="min-w-0 break-words">{entry.line}</span>
                    </div>
                  ))}
                  {isActive &&
                    systemLines.map((line, index) => (
                      <div key={`system-${index}`} className="flex gap-2 text-muted-foreground">
                        <span className="shrink-0 text-muted-foreground/60">--:--:--</span>
                        <span className="shrink-0 text-blue-600/80">[system]</span>
                        <span className="min-w-0 break-words">{line}</span>
                      </div>
                    ))}
                </>
              ) : isActive ? (
                systemLines.map((line, index) => (
                  <div key={`system-${index}`} className="flex gap-2 text-muted-foreground">
                    <span className="shrink-0 text-muted-foreground/60">--:--:--</span>
                    <span className="shrink-0 text-blue-600/80">[system]</span>
                    <span className="min-w-0 break-words">{line}</span>
                  </div>
                ))
              ) : (
                <p className="text-[11px] text-muted-foreground">
                  {t("tools.installConsoleEmptyRaw", "No log output")}
                </p>
              )}
              </div>
            </div>
          </ScrollArea>
        </div>
      )}
    </div>
  );
}
