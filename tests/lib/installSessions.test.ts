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
  const baseEvent = {
    toolId: "codex-desktop",
    toolName: "Codex (Desktop)",
    sessionId: "session-1",
    timestamp: "2026-05-09T12:00:00.000Z",
    level: "info" as const,
    kind: "session-started" as const,
    line: "Install session started",
  };

  beforeEach(() => {
    toolsApiMock.onInstallLog.mockReset();
  });

  it("creates a retained session from session-started and result events", () => {
    const sessionStarted = baseEvent;

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
    const firstSessionStarted = baseEvent;

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

  it("ignores late events from an older session even when they have a newer timestamp", () => {
    const firstSessionStarted = baseEvent;
    const secondSessionStarted = {
      ...firstSessionStarted,
      sessionId: "session-2",
      timestamp: "2026-05-09T12:05:00.000Z",
      line: "Retrying install session",
    };
    const lateOldSessionOutput = {
      ...firstSessionStarted,
      kind: "output" as const,
      level: "stdout" as const,
      timestamp: "2026-05-09T12:06:00.000Z",
      line: "Old session is still downloading",
    };

    const afterFirstSession = reduceInstallLogEvent(new Map(), firstSessionStarted);
    const afterSecondSession = reduceInstallLogEvent(
      afterFirstSession,
      secondSessionStarted,
    );
    const afterLateOldEvent = reduceInstallLogEvent(
      afterSecondSession,
      lateOldSessionOutput,
    );

    expect(afterLateOldEvent.get("codex-desktop")).toMatchObject({
      sessionId: "session-2",
      status: "running",
      startedAt: "2026-05-09T12:05:00.000Z",
      lastSummary: "Retrying install session",
    });
    expect(afterLateOldEvent.get("codex-desktop")?.entries).toHaveLength(1);
  });

  it("keeps lastSummary stable when raw output lines arrive", () => {
    const sessionStarted = baseEvent;

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

  it("promotes commands, meaningful phase changes, and meaningful output into activity", () => {
    const phase = {
      ...baseEvent,
      kind: "phase" as const,
      phase: "installing",
      timestamp: "2026-05-09T12:00:05.000Z",
      line: "Installing Codex CLI",
    };
    const phaseDuplicate = {
      ...phase,
      timestamp: "2026-05-09T12:00:06.000Z",
    };
    const command = {
      ...baseEvent,
      kind: "command" as const,
      timestamp: "2026-05-09T12:00:10.000Z",
      line: "Running installer command",
      command: "pnpm add -g @openai/codex",
    };
    const noisyOutput = {
      ...baseEvent,
      kind: "output" as const,
      level: "stdout" as const,
      timestamp: "2026-05-09T12:00:11.000Z",
      line: "1 package is looking for funding",
    };
    const meaningfulOutput = {
      ...baseEvent,
      kind: "output" as const,
      level: "stdout" as const,
      timestamp: "2026-05-09T12:00:12.000Z",
      line: "Download complete: shim created at C:\\Tools\\Codex",
    };

    const reduced = [
      baseEvent,
      phase,
      phaseDuplicate,
      command,
      noisyOutput,
      meaningfulOutput,
    ].reduce(reduceInstallLogEvent, new Map());

    expect(reduced.get("codex-desktop")?.activity).toEqual([
      {
        timestamp: "2026-05-09T12:00:05.000Z",
        kind: "phase",
        line: "Installing Codex CLI",
      },
      {
        timestamp: "2026-05-09T12:00:10.000Z",
        kind: "command",
        line: "Running installer command",
      },
      {
        timestamp: "2026-05-09T12:00:12.000Z",
        kind: "output",
        line: "Download complete: shim created at C:\\Tools\\Codex",
      },
    ]);
  });

  it("promotes meaningful output lines in common inflected verb forms", () => {
    const downloadingOutput = {
      ...baseEvent,
      kind: "output" as const,
      level: "stdout" as const,
      timestamp: "2026-05-09T12:00:08.000Z",
      line: "Downloading package metadata",
    };

    const reduced = [baseEvent, downloadingOutput].reduce(
      reduceInstallLogEvent,
      new Map(),
    );

    expect(reduced.get("codex-desktop")?.activity).toEqual([
      {
        timestamp: "2026-05-09T12:00:08.000Z",
        kind: "output",
        line: "Downloading package metadata",
      },
    ]);
  });

  it("collapses identical consecutive activity lines and keeps only the latest 3 items", () => {
    const events = [
      baseEvent,
      {
        ...baseEvent,
        kind: "output" as const,
        level: "stdout" as const,
        timestamp: "2026-05-09T12:00:01.000Z",
        line: "Downloading package",
      },
      {
        ...baseEvent,
        kind: "output" as const,
        level: "stdout" as const,
        timestamp: "2026-05-09T12:00:02.000Z",
        line: "Downloading package",
      },
      {
        ...baseEvent,
        kind: "command" as const,
        timestamp: "2026-05-09T12:00:03.000Z",
        line: "Running install command",
        command: "installer.exe /S",
      },
      {
        ...baseEvent,
        kind: "phase" as const,
        timestamp: "2026-05-09T12:00:04.000Z",
        phase: "configuring",
        line: "Configuring environment",
      },
      {
        ...baseEvent,
        kind: "output" as const,
        level: "stdout" as const,
        timestamp: "2026-05-09T12:00:05.000Z",
        line: "Version 1.2.3 installed successfully",
      },
    ].reduce(reduceInstallLogEvent, new Map());

    expect(events.get("codex-desktop")?.activity).toEqual([
      {
        timestamp: "2026-05-09T12:00:03.000Z",
        kind: "command",
        line: "Running install command",
      },
      {
        timestamp: "2026-05-09T12:00:04.000Z",
        kind: "phase",
        line: "Configuring environment",
      },
      {
        timestamp: "2026-05-09T12:00:05.000Z",
        kind: "output",
        line: "Version 1.2.3 installed successfully",
      },
    ]);
  });

  it("keeps consecutive promoted items with the same line when their kinds differ", () => {
    const phase = {
      ...baseEvent,
      kind: "phase" as const,
      phase: "installing",
      timestamp: "2026-05-09T12:00:05.000Z",
      line: "Install complete",
    };
    const output = {
      ...baseEvent,
      kind: "output" as const,
      level: "stdout" as const,
      timestamp: "2026-05-09T12:00:06.000Z",
      line: "Install complete",
    };

    const reduced = [baseEvent, phase, output].reduce(
      reduceInstallLogEvent,
      new Map(),
    );

    expect(reduced.get("codex-desktop")?.activity).toEqual([
      {
        timestamp: "2026-05-09T12:00:05.000Z",
        kind: "phase",
        line: "Install complete",
      },
      {
        timestamp: "2026-05-09T12:00:06.000Z",
        kind: "output",
        line: "Install complete",
      },
    ]);
  });

  it("promotes terminal results into activity and updates terminal session state", () => {
    const result = {
      ...baseEvent,
      kind: "result" as const,
      level: "error" as const,
      timestamp: "2026-05-09T12:00:30.000Z",
      line: "Install failed with exit code 1",
      exitCode: 1,
    };

    const reduced = [baseEvent, result].reduce(reduceInstallLogEvent, new Map());
    const session = reduced.get("codex-desktop");

    expect(session?.status).toBe("error");
    expect(session?.endedAt).toBe("2026-05-09T12:00:30.000Z");
    expect(session?.lastSummary).toBe("Install failed with exit code 1");
    expect(session?.activity).toEqual([
      {
        timestamp: "2026-05-09T12:00:30.000Z",
        kind: "result",
        line: "Install failed with exit code 1",
      },
    ]);
  });

  it("retains a terminal result in activity even when its line matches the previous promoted item", () => {
    const output = {
      ...baseEvent,
      kind: "output" as const,
      level: "stdout" as const,
      timestamp: "2026-05-09T12:00:20.000Z",
      line: "Install completed",
    };
    const result = {
      ...baseEvent,
      kind: "result" as const,
      level: "success" as const,
      timestamp: "2026-05-09T12:00:30.000Z",
      line: "Install completed",
      exitCode: 0,
    };

    const reduced = [baseEvent, output, result].reduce(
      reduceInstallLogEvent,
      new Map(),
    );

    expect(reduced.get("codex-desktop")?.activity).toEqual([
      {
        timestamp: "2026-05-09T12:00:20.000Z",
        kind: "output",
        line: "Install completed",
      },
      {
        timestamp: "2026-05-09T12:00:30.000Z",
        kind: "result",
        line: "Install completed",
      },
    ]);
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
