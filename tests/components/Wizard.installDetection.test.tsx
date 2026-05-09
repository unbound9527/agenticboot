import { QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Wizard } from "@/pages/Wizard";
import { createTestQueryClient } from "../utils/testQueryClient";

const TOOL_IDS = [
  "claude-code-cli",
  "claude-code-desktop",
  "codex-cli",
  "codex-desktop",
  "gemini-cli",
  "opencode-cli",
  "opencode-desktop",
  "openclaw",
  "hermes",
] as const;

const toolsApiMock = vi.hoisted(() => ({
  checkNetwork: vi.fn(),
  detectTools: vi.fn(),
  resolveInstallPlan: vi.fn(),
  executeInstallPlan: vi.fn(),
  uninstallTool: vi.fn(),
  getInstalledTools: vi.fn(),
  hasAnyInstalledTools: vi.fn(),
  getInstallRoot: vi.fn(),
  setInstallRoot: vi.fn(),
  checkToolUpdates: vi.fn(),
  onInstallProgress: vi.fn(),
  onInstallComplete: vi.fn(),
  onInstallError: vi.fn(),
}));

vi.mock("@/lib/api/tools", () => ({
  toolsApi: toolsApiMock,
}));

function buildDetectResults(installedIds: string[]) {
  const installed = new Set(installedIds);
  return TOOL_IDS.map((id) => ({
    installed: installed.has(id),
    version: installed.has(id) ? "1.0.0" : undefined,
    installPath: installed.has(id) ? `C:\\Tools\\${id}` : undefined,
  }));
}

function createDeferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe("Wizard install detection", () => {
  beforeEach(() => {
    toolsApiMock.checkNetwork.mockResolvedValue({
      githubReachable: true,
      npmReachable: true,
      youtubeReachable: true,
    });
    toolsApiMock.detectTools.mockReset();
    toolsApiMock.detectTools.mockResolvedValue(
      buildDetectResults(["claude-code-cli"]),
    );
    toolsApiMock.getInstallRoot.mockReset();
    toolsApiMock.getInstallRoot.mockResolvedValue(null);
    toolsApiMock.onInstallProgress.mockResolvedValue(() => {});
  });

  it("uses D:\\AgenticBoot by default when there is no saved install root", async () => {
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Wizard onComplete={vi.fn()} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticBoot",
      );
    });

    expect(screen.getByDisplayValue("D:\\AgenticBoot")).toHaveAttribute(
      "placeholder",
      "D:\\AgenticBoot",
    );
  });

  it("waits for the saved install root before starting initial detection", async () => {
    vi.useFakeTimers();

    try {
      const savedRoot = "E:\\SavedTools";
      const deferredRoot = createDeferred<string | null>();
      toolsApiMock.getInstallRoot.mockReturnValueOnce(deferredRoot.promise);

      render(
        <QueryClientProvider client={createTestQueryClient()}>
          <Wizard onComplete={vi.fn()} />
        </QueryClientProvider>,
      );

      await act(async () => {
        await vi.advanceTimersByTimeAsync(400);
      });
      expect(toolsApiMock.detectTools).not.toHaveBeenCalled();

      await act(async () => {
        deferredRoot.resolve(savedRoot);
        await Promise.resolve();
        await Promise.resolve();
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        savedRoot,
      );
      expect(toolsApiMock.detectTools).toHaveBeenCalledTimes(1);
      expect(screen.getByDisplayValue(savedRoot)).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });

  it("falls back to D:\\AgenticBoot when install-root loading hangs", async () => {
    vi.useFakeTimers();

    try {
      const deferredRoot = createDeferred<string | null>();
      toolsApiMock.getInstallRoot.mockReturnValueOnce(deferredRoot.promise);

      render(
        <QueryClientProvider client={createTestQueryClient()}>
          <Wizard onComplete={vi.fn()} />
        </QueryClientProvider>,
      );

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });
      expect(toolsApiMock.detectTools).not.toHaveBeenCalled();

      await act(async () => {
        await vi.advanceTimersByTimeAsync(1000);
      });

      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticBoot",
      );
      expect(toolsApiMock.detectTools).toHaveBeenCalledTimes(1);
      expect(screen.getByDisplayValue("D:\\AgenticBoot")).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });

  it("re-runs detection for a slow saved root after fallback and ignores stale earlier results", async () => {
    vi.useFakeTimers();

    try {
      const savedRoot = "E:\\SavedTools";
      const deferredRoot = createDeferred<string | null>();
      const defaultDetect = createDeferred<ReturnType<typeof buildDetectResults>>();
      const savedDetect = createDeferred<ReturnType<typeof buildDetectResults>>();

      toolsApiMock.getInstallRoot.mockReturnValueOnce(deferredRoot.promise);
      toolsApiMock.detectTools.mockImplementation((_toolIds, installRoot) => {
        if (installRoot === "D:\\AgenticBoot") {
          return defaultDetect.promise;
        }

        if (installRoot === savedRoot) {
          return savedDetect.promise;
        }

        return Promise.resolve(buildDetectResults(["claude-code-cli"]));
      });

      render(
        <QueryClientProvider client={createTestQueryClient()}>
          <Wizard onComplete={vi.fn()} />
        </QueryClientProvider>,
      );

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(600);
      });

      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticBoot",
      );
      expect(toolsApiMock.detectTools).toHaveBeenCalledTimes(1);

      await act(async () => {
        deferredRoot.resolve(savedRoot);
        await Promise.resolve();
        await Promise.resolve();
      });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(toolsApiMock.detectTools).toHaveBeenLastCalledWith(
        [...TOOL_IDS],
        savedRoot,
      );
      expect(screen.getByDisplayValue(savedRoot)).toBeInTheDocument();

      await act(async () => {
        savedDetect.resolve(buildDetectResults([]));
        await Promise.resolve();
        await Promise.resolve();
      });

      let checkboxes = screen.getAllByRole("checkbox");
      expect(checkboxes).toHaveLength(9);
      checkboxes.forEach((checkbox) => {
        expect(checkbox).toHaveAttribute("data-state", "checked");
      });

      await act(async () => {
        defaultDetect.resolve(buildDetectResults(["claude-code-cli"]));
        await Promise.resolve();
        await Promise.resolve();
      });

      checkboxes = screen.getAllByRole("checkbox");
      expect(checkboxes).toHaveLength(9);
      checkboxes.forEach((checkbox) => {
        expect(checkbox).toHaveAttribute("data-state", "checked");
      });
      expect(screen.getByDisplayValue(savedRoot)).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });

  it("re-runs detection when the install root changes and removes installed tools from selection", async () => {
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Wizard onComplete={vi.fn()} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticBoot",
      );
    });

    await waitFor(() => {
      expect(screen.getAllByRole("checkbox")).toHaveLength(8);
    });

    fireEvent.change(screen.getByDisplayValue("D:\\AgenticBoot"), {
      target: { value: "E:\\CustomTools" },
    });

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenLastCalledWith(
        [...TOOL_IDS],
        "E:\\CustomTools",
      );
    });
  });

  it("rebuilds the selected tools from the current install root detection", async () => {
    toolsApiMock.detectTools
      .mockResolvedValueOnce(buildDetectResults(["claude-code-cli"]))
      .mockResolvedValueOnce(buildDetectResults([]));

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Wizard onComplete={vi.fn()} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(screen.getAllByRole("checkbox")).toHaveLength(8);
    });
    fireEvent.click(screen.getByText("Codex (CLI)"));

    await waitFor(() => {
      const codexCheckbox = screen.getAllByRole("checkbox")[1];
      expect(codexCheckbox).toHaveAttribute("data-state", "unchecked");
    });

    fireEvent.change(screen.getByDisplayValue("D:\\AgenticBoot"), {
      target: { value: "E:\\CustomTools" },
    });

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenLastCalledWith(
        [...TOOL_IDS],
        "E:\\CustomTools",
      );
    });

    await waitFor(() => {
      const checkboxes = screen.getAllByRole("checkbox");
      expect(checkboxes).toHaveLength(9);
      checkboxes.forEach((checkbox) => {
        expect(checkbox).toHaveAttribute("data-state", "checked");
      });
    });
  });

  it("toggle all uses the current root available tool list after detection changes", async () => {
    toolsApiMock.detectTools
      .mockResolvedValueOnce(buildDetectResults(["claude-code-cli"]))
      .mockResolvedValueOnce(buildDetectResults(["gemini-cli"]));

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Wizard onComplete={vi.fn()} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(screen.getAllByRole("checkbox")).toHaveLength(8);
    });

    fireEvent.change(screen.getByDisplayValue("D:\\AgenticBoot"), {
      target: { value: "E:\\CustomTools" },
    });

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenLastCalledWith(
        [...TOOL_IDS],
        "E:\\CustomTools",
      );
    });

    const toggleAllButton = await screen.findByRole("button", {
      name: "全部取消",
    });
    fireEvent.click(toggleAllButton);
    fireEvent.click(await screen.findByRole("button", { name: "全部勾选" }));

    await waitFor(() => {
      const checkboxes = screen.getAllByRole("checkbox");
      expect(checkboxes).toHaveLength(8);
      checkboxes.forEach((checkbox) => {
        expect(checkbox).toHaveAttribute("data-state", "checked");
      });
    });
  });
});
