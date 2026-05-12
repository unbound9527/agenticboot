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
    const isNewerSessionStart =
      event.kind === "session-started" &&
      Date.parse(event.timestamp) >= Date.parse(current.startedAt);

    if (!isNewerSessionStart) {
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
          startedAt: event.timestamp,
          entries: [],
          activity: [],
        }
      : current;

  const nextActivity = appendActivityItem(
    base.activity,
    getPromotedActivityItem(event, base.activity),
  );

  next.set(event.toolId, {
    ...base,
    status:
      event.kind === "result" && event.level === "success"
        ? "complete"
        : event.kind === "result" && event.level === "error"
        ? "error"
          : base.status,
    endedAt: event.kind === "result" ? event.timestamp : base.endedAt,
    lastSummary: SUMMARY_EVENT_KINDS.has(event.kind)
      ? event.line
      : base.lastSummary,
    entries: [...base.entries, event],
    activity: nextActivity,
  });

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

  return sessions;
}
