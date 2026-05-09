import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const toolsApiMock = vi.hoisted(() => ({
  onInstallLog: vi.fn(),
}));

vi.mock("@/lib/api/tools", () => ({
  toolsApi: toolsApiMock,
}));

import {
  reduceInstallLogEvent,
  useInstallSessions,
} from "@/hooks/useInstallSessions";

describe("install session reducer", () => {
  beforeEach(() => {
    toolsApiMock.onInstallLog.mockReset();
  });

  it("creates a retained session from session-started and result events", () => {
    const sessionStarted = {
      toolId: "codex-desktop",
      toolName: "Codex (Desktop)",
      sessionId: "session-1",
      timestamp: "2026-05-09T12:00:00.000Z",
      level: "info" as const,
      kind: "session-started" as const,
      line: "Install session started",
    };

    const result = {
      ...sessionStarted,
      kind: "result" as const,
      level: "success" as const,
      line: "Install completed",
      exitCode: 0,
    };

    const started = reduceInstallLogEvent(new Map(), sessionStarted);
    const completed = reduceInstallLogEvent(started, result);

    expect(completed.get("codex-desktop")?.status).toBe("complete");
    expect(completed.get("codex-desktop")?.entries).toHaveLength(2);
  });

  it("cleans up a late-resolving install log subscription after unmount", async () => {
    let resolveSubscription: ((unlisten: () => void) => void) | undefined;
    const unlisten = vi.fn();

    toolsApiMock.onInstallLog.mockImplementation(
      () =>
        new Promise<() => void>((resolve) => {
          resolveSubscription = resolve;
        }),
    );

    const { unmount } = renderHook(() => useInstallSessions());

    unmount();
    resolveSubscription?.(unlisten);

    await waitFor(() => {
      expect(unlisten).toHaveBeenCalledTimes(1);
    });
  });
});
