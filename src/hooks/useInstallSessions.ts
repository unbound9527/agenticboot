import { useEffect, useState } from "react";
import { toolsApi } from "@/lib/api/tools";
import type {
  InstallActivityItem,
  InstallLogEvent,
  ToolInstallSession,
} from "@/types/tools";

const SUMMARY_EVENT_KINDS = new Set<InstallLogEvent["kind"]>([
  "session-started",
  "phase",
  "command",
  "result",
]);

const ACTIVITY_OUTPUT_PATTERN =
  /\b(download(?:ing|ed)?|install(?:ing|ed)?|config(?:ure|uring|ured)?|retry(?:ing|ied)?|wait(?:ing|ed)?|path|version|complete(?:d)?|created|failed|error|success(?:ful(?:ly)?)?|finaliz(?:e|ing|ed)|shim)\b/i;

function getPromotedActivityItem(
  event: InstallLogEvent,
  previousActivity: InstallActivityItem[],
): InstallActivityItem | null {
  if (event.kind === "command" || event.kind === "result") {
    return {
      timestamp: event.timestamp,
      kind: event.kind,
      line: event.line,
    };
  }

  if (event.kind === "phase") {
    const previousPhase = [...previousActivity]
      .reverse()
      .find((item) => item.kind === "phase");

    if (previousPhase?.line === event.line) {
      return null;
    }

    return {
      timestamp: event.timestamp,
      kind: "phase",
      line: event.line,
    };
  }

  if (event.kind === "output" && ACTIVITY_OUTPUT_PATTERN.test(event.line)) {
    return {
      timestamp: event.timestamp,
      kind: "output",
      line: event.line,
    };
  }

  return null;
}

function appendActivityItem(
  activity: InstallActivityItem[],
  item: InstallActivityItem | null,
): InstallActivityItem[] {
  if (!item) {
    return activity;
  }

  const previousItem = activity.at(-1);

  if (
    item.kind !== "result" &&
    previousItem?.kind === item.kind &&
    previousItem.line === item.line
  ) {
    return activity;
  }

  return [...activity, item].slice(-3);
}

function nowTimestamp() {
  return new Date().toISOString();
}

function buildOptimisticLogEvent(
  toolId: string,
  toolName: string,
  sessionId: string,
  line: string,
  kind: InstallLogEvent["kind"] = "phase",
): InstallLogEvent {
  return {
    toolId,
    toolName,
    sessionId,
    timestamp: nowTimestamp(),
    phase: "starting",
    level: "info",
    kind,
    line,
    source: "optimistic",
  };
}

function appendInstallLogEvent(
  session: ToolInstallSession,
  event: InstallLogEvent,
): ToolInstallSession {
  const nextActivity = appendActivityItem(
    session.activity,
    getPromotedActivityItem(event, session.activity),
  );

  return {
    ...session,
    status:
      event.kind === "result" && event.level === "success"
        ? "complete"
        : event.kind === "result" && event.level === "error"
          ? "error"
          : session.status,
    endedAt: event.kind === "result" ? event.timestamp : session.endedAt,
    lastSummary: SUMMARY_EVENT_KINDS.has(event.kind)
      ? event.line
      : session.lastSummary,
    entries: [...session.entries, event],
    activity: nextActivity,
  };
}

export function reduceInstallLogEvent(
  previous: Map<string, ToolInstallSession>,
  event: InstallLogEvent,
): Map<string, ToolInstallSession> {
  const next = new Map(previous);
  const current = next.get(event.toolId);

  if (
    current &&
    current.sessionId !== event.sessionId
  ) {
    const isOptimisticHandoff =
      current.source === "optimistic" && event.kind === "session-started";
    const isNewerSessionStart =
      event.kind === "session-started" &&
      Date.parse(event.timestamp) >= Date.parse(current.startedAt);

    if (!isNewerSessionStart && !isOptimisticHandoff) {
      return next;
    }
  }

  const base: ToolInstallSession =
    !current || current.sessionId !== event.sessionId
        ? {
          toolId: event.toolId,
          toolName: event.toolName,
          sessionId: event.sessionId,
          status: "running",
          source: event.source ?? "native",
          startedAt: event.timestamp,
          entries:
            current?.source === "optimistic" && event.kind === "session-started"
              ? current.entries
              : [],
          activity:
            current?.source === "optimistic" && event.kind === "session-started"
              ? current.activity
              : [],
          lastSummary:
            current?.source === "optimistic" && event.kind === "session-started"
              ? current.lastSummary
              : undefined,
        }
      : current;

  next.set(event.toolId, appendInstallLogEvent(base, event));

  return next;
}

export function useInstallSessions() {
  const [sessions, setSessions] = useState<Map<string, ToolInstallSession>>(
    () => new Map(),
  );

  useEffect(() => {
    let isMounted = true;
    let unlisten: (() => void) | undefined;

    toolsApi
      .onInstallLog((event) => {
        setSessions((previous) => reduceInstallLogEvent(previous, event));
      })
      .then((fn) => {
        if (!isMounted) {
          fn();
          return;
        }

        unlisten = fn;
      })
      .catch(() => {
        // Ignore subscription failures here so the hook does not surface
        // an unhandled rejection during mount/unmount transitions.
      });

    return () => {
      isMounted = false;
      unlisten?.();
    };
  }, []);

  const startOptimisticSession = (
    toolId: string,
    toolName: string,
    lines: string[],
  ) => {
    setSessions((previous) => {
      const next = new Map(previous);
      const sessionId = `optimistic-${toolId}`;
      const startedAt = nowTimestamp();
      let session: ToolInstallSession = {
        toolId,
        toolName,
        sessionId,
        status: "running",
        source: "optimistic",
        startedAt,
        entries: [],
        activity: [],
      };

      for (const line of lines) {
        session = appendInstallLogEvent(
          session,
          buildOptimisticLogEvent(toolId, toolName, sessionId, line),
        );
      }

      next.set(toolId, session);
      return next;
    });
  };

  const appendOptimisticEntry = (
    toolId: string,
    toolName: string,
    line: string,
    kind: InstallLogEvent["kind"] = "phase",
  ) => {
    setSessions((previous) => {
      const next = new Map(previous);
      const current = next.get(toolId);
      if (!current || current.status !== "running") {
        return previous;
      }

      next.set(
        toolId,
        appendInstallLogEvent(
          current,
          buildOptimisticLogEvent(toolId, toolName, current.sessionId, line, kind),
        ),
      );
      return next;
    });
  };

  const markSessionError = (toolId: string, toolName: string, line: string) => {
    setSessions((previous) => {
      const next = new Map(previous);
      const current = next.get(toolId);
      if (!current) {
        return previous;
      }

      next.set(
        toolId,
        appendInstallLogEvent(current, {
          toolId,
          toolName,
          sessionId: current.sessionId,
          timestamp: nowTimestamp(),
          phase: "error",
          level: "error",
          kind: "result",
          line,
          exitCode: null,
          source: current.source ?? "optimistic",
        }),
      );
      return next;
    });
  };

  return {
    sessions,
    startOptimisticSession,
    appendOptimisticEntry,
    markSessionError,
  };
}
