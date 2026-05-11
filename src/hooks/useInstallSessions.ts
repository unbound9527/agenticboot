import { useEffect, useState } from "react";
import { toolsApi } from "@/lib/api/tools";
import type { InstallLogEvent, ToolInstallSession } from "@/types/tools";

const SUMMARY_EVENT_KINDS = new Set<InstallLogEvent["kind"]>([
  "session-started",
  "phase",
  "command",
  "result",
]);

export function reduceInstallLogEvent(
  previous: Map<string, ToolInstallSession>,
  event: InstallLogEvent,
): Map<string, ToolInstallSession> {
  const next = new Map(previous);
  const current = next.get(event.toolId);

  if (
    current &&
    current.sessionId !== event.sessionId &&
    Date.parse(event.timestamp) < Date.parse(current.startedAt)
  ) {
    return next;
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
        }
      : current;

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
