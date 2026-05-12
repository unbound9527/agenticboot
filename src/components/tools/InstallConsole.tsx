import { useState, useEffect } from "react";
import { TerminalSquare, ChevronDown, ChevronUp, Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { InstallActivityItem, InstallLogEvent, ToolInstallSession } from "@/types/tools";

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

function getPhaseColor(phase: string): string {
  switch (phase) {
    case "downloading": return "text-blue-500";
    case "extracting": return "text-yellow-500";
    case "installing": return "text-purple-500";
    case "configuring": return "text-green-500";
    case "complete": return "text-emerald-500";
    case "error": return "text-red-500";
    default: return "text-muted-foreground";
  }
}

function getActivityKindLabel(kind: string): string {
  switch (kind) {
    case "command":
      return "[command]";
    case "output":
      return "[output]";
    case "result":
      return "[result]";
    case "phase":
      return "[phase]";
    default:
      return `[${kind}]`;
  }
}

function getActivityKindColor(kind: string): string {
  switch (kind) {
    case "command":
      return "text-blue-500";
    case "output":
      return "text-muted-foreground";
    case "result":
      return "text-emerald-500";
    default:
      return "text-foreground";
  }
}

function renderSummaryPreview(entry: InstallLogEvent) {
  return (
    <span className={`text-[11px] ${getPhaseColor(entry.phase ?? "")}`}>
      {entry.phase ? `[${entry.phase}] ` : ""}
      {entry.line}
    </span>
  );
}

function renderActivityPreview(entry: InstallActivityItem) {
  return (
    <span className={`text-[11px] ${getActivityKindColor(entry.kind)}`}>
      {getActivityKindLabel(entry.kind)} {entry.line}
    </span>
  );
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
  const [selectedTab, setSelectedTab] = useState("summary");

  const summaryEntries = session?.entries.filter((entry) => entry.kind !== "output") ?? [];
  const activityEntries = session?.activity ?? [];
  const latestActivity = activityEntries[activityEntries.length - 1];
  const latestSummaryEntry = summaryEntries[summaryEntries.length - 1];
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
    setSelectedTab("summary");
  }, [session?.sessionId]);

  if (!session) {
    return null;
  }

  return (
    <div className="overflow-hidden rounded-lg border border-border/60 bg-background shadow-sm">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex w-full items-center justify-between border-b border-border/60 bg-muted/50 px-4 py-2 hover:bg-muted/70 transition-colors"
      >
        <div className="flex items-center gap-2">
          {isActive ? (
            <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground" />
          ) : (
            <TerminalSquare className="h-3.5 w-3.5 text-muted-foreground" />
          )}
          <span className="text-[12px] font-medium truncate">{session.toolName}</span>
          <Badge
            variant="secondary"
            className={`text-[10px] px-1.5 py-0 ${
              session.status === "complete" ? "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-400" :
              session.status === "error" ? "bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-400" :
              ""
            }`}
          >
            {session.status}
          </Badge>
          {latestActivity ? (
            renderActivityPreview(latestActivity)
          ) : latestSummaryEntry ? (
            renderSummaryPreview(latestSummaryEntry)
          ) : isActive ? (
            <span className="flex items-center gap-1 text-[11px] text-muted-foreground">
              <CyclingChar isAnimating={true} className="text-muted-foreground" />
            </span>
          ) : null}
        </div>
        {expanded ? (
          <ChevronUp className="h-4 w-4 text-muted-foreground" />
        ) : (
          <ChevronDown className="h-4 w-4 text-muted-foreground" />
        )}
      </button>

      {expanded && (
        <>
          {activityEntries.length > 0 && (
            <div className="border-b border-border/60 px-4 py-3">
              <div className="mb-2 flex items-center justify-between gap-3">
                <span className="text-[11px] font-medium uppercase tracking-[0.08em] text-muted-foreground">
                  {t("tools.installConsoleActivity", "最近活动")}
                </span>
                <span className="text-[10px] text-muted-foreground">
                  {t("tools.installConsoleItemsCount", "{{count}} 条", {
                    count: activityEntries.length,
                  })}
                </span>
              </div>
              <div className="space-y-1.5 font-mono text-[11px] leading-4">
                {activityEntries.map((entry, index) => (
                  <div key={`${entry.timestamp}-${index}`} className="flex gap-2">
                    <span className="shrink-0 text-muted-foreground/60">
                      {formatTimestamp(entry.timestamp)}
                    </span>
                    <span className={`shrink-0 ${getActivityKindColor(entry.kind)}`}>
                      {getActivityKindLabel(entry.kind)}
                    </span>
                    <span className="min-w-0 break-words text-foreground">{entry.line}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
          <Tabs value={selectedTab} onValueChange={setSelectedTab} className="w-full">
            <div className="flex items-center justify-between border-b border-border/60 px-4 pt-2">
              <TabsList className="h-7">
                <TabsTrigger value="summary" className="text-[11px] py-1">
                  {t("tools.installConsoleSummary", "摘要")}
                </TabsTrigger>
                <TabsTrigger value="raw" className="text-[11px] py-1">
                  {t("tools.installConsoleRaw", "原始输出")}
                </TabsTrigger>
              </TabsList>
              <span className="text-[10px] text-muted-foreground">
                {t("tools.installConsoleStarted", "开始时间")}: {formatTimestamp(session.startedAt)}
              </span>
            </div>

            <TabsContent value="summary" className="mt-0 p-3">
              <ScrollArea className="max-h-48">
                <div className="space-y-1 font-mono text-[11px] leading-4">
                  {summaryEntries.length > 0 ? (
                    summaryEntries.map((entry, index) => (
                      <div key={`${entry.timestamp}-${index}`} className="flex gap-2">
                        <span className={`shrink-0 ${getPhaseColor(entry.phase ?? "")}`}>
                          {entry.phase ? `[${entry.phase}]` : `[${entry.kind}]`}
                        </span>
                        <span className="min-w-0 break-words text-foreground">{entry.line}</span>
                      </div>
                    ))
                  ) : isActive ? (
                    <div className="flex items-center gap-2 text-muted-foreground">
                      <CyclingChar isAnimating={true} className="text-muted-foreground" />
                      <span className="text-[11px]">
                        {t("tools.consoleWaiting", "初始化中...")}
                      </span>
                    </div>
                  ) : (
                    <p className="text-[11px] text-muted-foreground">
                      {t("tools.installConsoleEmpty", "暂无摘要信息")}
                    </p>
                  )}
                </div>
              </ScrollArea>
            </TabsContent>

            <TabsContent value="raw" className="mt-0">
              <ScrollArea className="max-h-48">
                <div className="space-y-1 px-3 py-2 font-mono text-[11px] leading-4">
                  {session.entries.length > 0 ? (
                    session.entries.map((entry, index) => (
                      <div key={`${entry.timestamp}-${index}`} className="flex gap-2">
                        <span className="shrink-0 text-muted-foreground/60">
                          {formatTimestamp(entry.timestamp)}
                        </span>
                        <span className={`shrink-0 ${
                          entry.level === "error" || entry.level === "stderr" ? "text-red-500" :
                          entry.kind === "output" ? "text-muted-foreground" :
                          "text-foreground"
                        }`}>
                          [{entry.kind}]
                        </span>
                        <span className="min-w-0 break-words">{entry.line}</span>
                      </div>
                    ))
                  ) : isActive ? (
                    <div className="flex items-center gap-2 text-muted-foreground">
                      <CyclingChar isAnimating={true} className="text-muted-foreground" />
                      <span className="text-[11px]">
                        {t("tools.consoleWaiting", "初始化中...")}
                      </span>
                    </div>
                  ) : (
                    <p className="text-muted-foreground text-[11px]">
                      {t("tools.installConsoleEmptyRaw", "暂无日志输出")}
                    </p>
                  )}
                </div>
              </ScrollArea>
            </TabsContent>
          </Tabs>
        </>
      )}
    </div>
  );
}
