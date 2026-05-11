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

  it("ignores stale late events from an older session after a newer session starts", () => {
    const firstSessionStarted = {
      toolId: "codex-desktop",
      toolName: "Codex (Desktop)",
      sessionId: "session-1",
      timestamp: "2026-05-09T12:00:00.000Z",
      level: "info" as const,
      kind: "session-started" as const,
      line: "Install session started",
    };

    const secondSessionStarted = {
      ...firstSessionStarted,
      sessionId: "session-2",
      timestamp: "2026-05-09T12:05:00.000Z",
      line: "Retrying install session",
    };

    const staleResult = {
      ...firstSessionStarted,
      kind: "result" as const,
      level: "error" as const,
      line: "Old session failed late",
      exitCode: 1,
    };

    const afterFirstSession = reduceInstallLogEvent(new Map(), firstSessionStarted);
    const afterSecondSession = reduceInstallLogEvent(
      afterFirstSession,
      secondSessionStarted,
    );
    const afterStaleResult = reduceInstallLogEvent(afterSecondSession, staleResult);

    expect(afterStaleResult.get("codex-desktop")?.sessionId).toBe("session-2");
    expect(afterStaleResult.get("codex-desktop")?.status).toBe("running");
    expect(afterStaleResult.get("codex-desktop")?.entries).toHaveLength(1);
    expect(afterStaleResult.get("codex-desktop")?.lastSummary).toBe(
      "Retrying install session",
    );
  });

  it("keeps lastSummary stable when raw output lines arrive", () => {
    const sessionStarted = {
      toolId: "codex-desktop",
      toolName: "Codex (Desktop)",
      sessionId: "session-1",
      timestamp: "2026-05-09T12:00:00.000Z",
      level: "info" as const,
      kind: "session-started" as const,
      line: "Install session started",
    };

    const output = {
      ...sessionStarted,
      kind: "output" as const,
      level: "stdout" as const,
      line: "Downloading package chunk 1/4",
    };

    const started = reduceInstallLogEvent(new Map(), sessionStarted);
    const withOutput = reduceInstallLogEvent(started, output);

    expect(withOutput.get("codex-desktop")?.lastSummary).toBe(
      "Install session started",
    );
    expect(withOutput.get("codex-desktop")?.entries).toHaveLength(2);
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

  it("handles install log subscription rejection without an unhandled failure", async () => {
    toolsApiMock.onInstallLog.mockRejectedValue(new Error("subscription failed"));

    renderHook(() => useInstallSessions());

    await waitFor(() => {
      expect(toolsApiMock.onInstallLog).toHaveBeenCalledTimes(1);
    });
  });
});
