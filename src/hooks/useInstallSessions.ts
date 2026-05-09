import { useEffect, useState } from "react";
import { toolsApi } from "@/lib/api/tools";
import type { InstallLogEvent, ToolInstallSession } from "@/types/tools";

export function reduceInstallLogEvent(
  previous: Map<string, ToolInstallSession>,
  event: InstallLogEvent,
): Map<string, ToolInstallSession> {
  const next = new Map(previous);
  const current = next.get(event.toolId);

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
    lastSummary: event.line,
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
      });

    return () => {
      isMounted = false;
      unlisten?.();
    };
  }, []);

  return sessions;
}
