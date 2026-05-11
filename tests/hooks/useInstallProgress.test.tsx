import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useInstallProgress } from "@/hooks/useInstallProgress";

const toolsApiMock = vi.hoisted(() => ({
  onInstallProgress: vi.fn(),
}));

let installProgressListener:
  | ((progress: {
      toolId: string;
      toolName: string;
      phase:
        | "starting"
        | "downloading"
        | "extracting"
        | "installing"
        | "configuring"
        | "complete"
        | "error"
        | "skipped";
      percent: number;
      message: string;
    }) => void)
  | null = null;

vi.mock("@/lib/api/tools", () => ({
  toolsApi: toolsApiMock,
}));

describe("useInstallProgress", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    installProgressListener = null;
    toolsApiMock.onInstallProgress.mockReset();
    toolsApiMock.onInstallProgress.mockImplementation(async (callback) => {
      installProgressListener = callback;
      return () => {
        installProgressListener = null;
      };
    });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("smoothly increments displayed progress toward the latest active percent", async () => {
    const { result } = renderHook(() => useInstallProgress());

    await act(async () => {
      await Promise.resolve();
    });
    expect(toolsApiMock.onInstallProgress).toHaveBeenCalledTimes(1);

    act(() => {
      installProgressListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        phase: "installing",
        percent: 65,
        message: "Installing OpenClaw",
      });
    });

    expect(result.current.getToolProgress("openclaw")?.phase).toBe("installing");
    expect(result.current.getToolProgress("openclaw")?.percent).toBeLessThan(65);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(200);
    });

    const midProgress = result.current.getToolProgress("openclaw")?.percent ?? 0;
    expect(midProgress).toBeGreaterThan(0);
    expect(midProgress).toBeLessThan(65);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(2000);
    });

    expect(result.current.getToolProgress("openclaw")?.percent).toBe(65);
  });

  it("snaps terminal states to their final percent immediately", async () => {
    const { result } = renderHook(() => useInstallProgress());

    await act(async () => {
      await Promise.resolve();
    });
    expect(toolsApiMock.onInstallProgress).toHaveBeenCalledTimes(1);

    act(() => {
      installProgressListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        phase: "installing",
        percent: 65,
        message: "Installing OpenClaw",
      });
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(100);
    });

    act(() => {
      installProgressListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        phase: "complete",
        percent: 100,
        message: "OpenClaw installed",
      });
    });

    expect(result.current.getToolProgress("openclaw")).toMatchObject({
      phase: "complete",
      percent: 100,
      message: "OpenClaw installed",
    });
  });
});
