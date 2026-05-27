import { QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
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

const TOOL_NAMES: Record<(typeof TOOL_IDS)[number], string> = {
  "claude-code-cli": "Claude Code (CLI)",
  "claude-code-desktop": "Claude Code (桌面版)",
  "codex-cli": "Codex (CLI)",
  "codex-desktop": "Codex (桌面版)",
  "gemini-cli": "Gemini CLI",
  "opencode-cli": "OpenCode (CLI)",
  "opencode-desktop": "OpenCode (桌面版)",
  openclaw: "OpenClaw",
  hermes: "Hermes (Web UI)",
};

function buildToolCatalog() {
  return TOOL_IDS.map((id) => ({
    id,
    name: TOOL_NAMES[id],
    description: `${TOOL_NAMES[id]} description`,
    icon: id,
    category: "ai-cli",
    installStrategy: id.includes("desktop") ? "desktop-installer" : "global-npm",
    dependencies: [],
    updateSource: undefined,
    platformSupport: {
      windows: "implemented",
      macos: "planned",
      linux: "planned",
    },
    capabilities: {
      canInstall: true,
      canUninstall: true,
      canLaunch: id.includes("desktop"),
      canUpdate: false,
      supportsPathlessUninstall: id.includes("desktop"),
      commandName: id,
      managedShimName: id,
      managedExecutableCandidates: [],
    },
  }));
}

function buildCatalogItem(
  id: (typeof TOOL_IDS)[number],
  overrides: Partial<ReturnType<typeof buildToolCatalog>[number]> = {},
) {
  return {
    ...buildToolCatalog().find((tool) => tool.id === id)!,
    ...overrides,
  };
}

const toolsApiMock = vi.hoisted(() => ({
  checkNetwork: vi.fn(),
  getToolCatalog: vi.fn(),
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
  onInstallLog: vi.fn(),
  onInstallComplete: vi.fn(),
  onInstallError: vi.fn(),
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

let installLogListener:
  | ((event: {
      toolId: string;
      toolName: string;
      sessionId: string;
      timestamp: string;
      phase?: string;
      level: "info" | "stdout" | "stderr" | "success" | "error";
      kind: "session-started" | "phase" | "command" | "output" | "result";
      line: string;
      command?: string;
      exitCode?: number | null;
    }) => void)
  | null = null;

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

function expectCheckedToolIds(expectedCheckedIds: string[]) {
  const expected = new Set(expectedCheckedIds);

  TOOL_IDS.forEach((id) => {
    const card = screen.getByText(TOOL_NAMES[id]).closest("[class*='rounded-lg']");
    expect(card).not.toBeNull();

    const checkbox = within(card as HTMLElement).queryByRole("checkbox");
    if (!checkbox) {
      return;
    }

    expect(checkbox).toHaveAttribute(
      "data-state",
      expected.has(id) ? "checked" : "unchecked",
    );
  });
}

function getCheckboxForTool(id: (typeof TOOL_IDS)[number]) {
  const card = screen.getByText(TOOL_NAMES[id]).closest("[class*='rounded-lg']");
  expect(card).not.toBeNull();
  const checkbox = within(card as HTMLElement).queryByRole("checkbox");
  expect(checkbox).not.toBeNull();
  return checkbox as HTMLElement;
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
    toolsApiMock.getToolCatalog.mockResolvedValue(buildToolCatalog());
    toolsApiMock.checkNetwork.mockResolvedValue({
      githubReachable: true,
      npmReachable: true,
      youtubeReachable: true,
    });
    toolsApiMock.getInstallRoot.mockResolvedValue(null);
    toolsApiMock.detectTools.mockReset();
    toolsApiMock.detectTools.mockResolvedValue(
      buildDetectResults(["claude-code-cli"]),
    );
    toolsApiMock.resolveInstallPlan.mockReset();
    toolsApiMock.executeInstallPlan.mockReset();
    toolsApiMock.getInstallRoot.mockReset();
    toolsApiMock.getInstallRoot.mockResolvedValue(null);
    installProgressListener = null;
    installLogListener = null;
    toolsApiMock.onInstallProgress.mockImplementation(async (callback) => {
      installProgressListener = callback;
      return () => {
        installProgressListener = null;
      };
    });
    toolsApiMock.onInstallLog.mockImplementation(async (callback) => {
      installLogListener = callback;
      return () => {
        installLogListener = null;
      };
    });
  });

  it("uses D:\\AgenticTools by default when there is no saved install root", async () => {
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Wizard onComplete={vi.fn()} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        false,
      );
    });

    expect(screen.getByDisplayValue("D:\\AgenticTools")).toHaveAttribute(
      "placeholder",
      "D:\\AgenticTools",
    );

    expectCheckedToolIds([
      "claude-code-desktop",
      "codex-desktop",
    ]);
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
        false,
      );
      expect(toolsApiMock.detectTools).toHaveBeenCalledTimes(1);
      expect(screen.getByDisplayValue(savedRoot)).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });

  it("falls back to D:\\AgenticTools when install-root loading hangs", async () => {
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
        "D:\\AgenticTools",
        false,
      );
      expect(toolsApiMock.detectTools).toHaveBeenCalledTimes(1);
      expect(screen.getByDisplayValue("D:\\AgenticTools")).toBeInTheDocument();
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
        if (installRoot === "D:\\AgenticTools") {
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
        "D:\\AgenticTools",
        false,
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
        false,
      );
      expect(screen.getByDisplayValue(savedRoot)).toBeInTheDocument();

      await act(async () => {
        savedDetect.resolve(buildDetectResults([]));
        await Promise.resolve();
        await Promise.resolve();
      });

      expectCheckedToolIds([
        "claude-code-desktop",
        "codex-desktop",
      ]);

      await act(async () => {
        defaultDetect.resolve(buildDetectResults(["claude-code-cli"]));
        await Promise.resolve();
        await Promise.resolve();
      });

      expectCheckedToolIds([
        "claude-code-desktop",
        "codex-desktop",
      ]);
      expect(screen.getByDisplayValue(savedRoot)).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });

  it("ignores fallback detection results that resolve after a late saved root arrives but before replacement detection starts", async () => {
    vi.useFakeTimers();

    try {
      const savedRoot = "E:\\SavedTools";
      const deferredRoot = createDeferred<string | null>();
      const defaultDetect = createDeferred<ReturnType<typeof buildDetectResults>>();
      const savedDetect = createDeferred<ReturnType<typeof buildDetectResults>>();

      toolsApiMock.getInstallRoot.mockReturnValueOnce(deferredRoot.promise);
      toolsApiMock.detectTools.mockImplementation((_toolIds, installRoot) => {
        if (installRoot === "D:\\AgenticTools") {
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
        "D:\\AgenticTools",
        false,
      );
      expect(toolsApiMock.detectTools).toHaveBeenCalledTimes(1);

      await act(async () => {
        deferredRoot.resolve(savedRoot);
        await Promise.resolve();
        await Promise.resolve();
      });

      expect(screen.getByDisplayValue(savedRoot)).toBeInTheDocument();
      expect(toolsApiMock.detectTools).toHaveBeenCalledTimes(1);

      await act(async () => {
        defaultDetect.resolve(buildDetectResults(["claude-code-cli"]));
        await Promise.resolve();
        await Promise.resolve();
      });

      const refreshButton = screen.getByRole("button", { name: "重新检测" });
      expect(refreshButton).toBeDisabled();

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(toolsApiMock.detectTools).toHaveBeenLastCalledWith(
        [...TOOL_IDS],
        savedRoot,
        false,
      );

      await act(async () => {
        savedDetect.resolve(buildDetectResults([]));
        await Promise.resolve();
        await Promise.resolve();
      });

      expectCheckedToolIds([
        "claude-code-desktop",
        "codex-desktop",
      ]);
    } finally {
      vi.useRealTimers();
    }
  });

  it("does not let a late saved root overwrite a user-typed install root", async () => {
    vi.useFakeTimers();

    try {
      const savedRoot = "F:\\SavedTools";
      const userRoot = "E:\\MyTools";
      const deferredRoot = createDeferred<string | null>();
      toolsApiMock.getInstallRoot.mockReturnValueOnce(deferredRoot.promise);

      render(
        <QueryClientProvider client={createTestQueryClient()}>
          <Wizard onComplete={vi.fn()} />
        </QueryClientProvider>,
      );

      fireEvent.change(screen.getByDisplayValue("D:\\AgenticTools"), {
        target: { value: userRoot },
      });

      await act(async () => {
        deferredRoot.resolve(savedRoot);
        await Promise.resolve();
        await Promise.resolve();
      });

      expect(screen.getByDisplayValue(userRoot)).toBeInTheDocument();

      await act(async () => {
        await vi.advanceTimersByTimeAsync(500);
      });

      expect(toolsApiMock.detectTools).toHaveBeenLastCalledWith(
        [...TOOL_IDS],
        userRoot,
        false,
      );
      expect(toolsApiMock.detectTools).not.toHaveBeenCalledWith(
        [...TOOL_IDS],
        savedRoot,
        false,
      );
      expect(screen.getByDisplayValue(userRoot)).toBeInTheDocument();
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
        "D:\\AgenticTools",
        false,
      );
    });

    await waitFor(() => {
      expect(screen.getAllByRole("checkbox")).toHaveLength(8);
    });

    fireEvent.change(screen.getByDisplayValue("D:\\AgenticTools"), {
      target: { value: "E:\\CustomTools" },
    });

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenLastCalledWith(
        [...TOOL_IDS],
        "E:\\CustomTools",
        false,
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
    fireEvent.click(screen.getByText(TOOL_NAMES["codex-desktop"]));

    await waitFor(() => {
      const codexCheckbox = getCheckboxForTool("codex-desktop");
      expect(codexCheckbox).toHaveAttribute("data-state", "unchecked");
    });

    fireEvent.change(screen.getByDisplayValue("D:\\AgenticTools"), {
      target: { value: "E:\\CustomTools" },
    });

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenLastCalledWith(
        [...TOOL_IDS],
        "E:\\CustomTools",
        false,
      );
    });

    await waitFor(() => {
      expectCheckedToolIds([
        "claude-code-desktop",
        "codex-desktop",
      ]);
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

    fireEvent.change(screen.getByDisplayValue("D:\\AgenticTools"), {
      target: { value: "E:\\CustomTools" },
    });

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenLastCalledWith(
        [...TOOL_IDS],
        "E:\\CustomTools",
        false,
      );
    });

    const selectAllButton = await screen.findByRole("button", {
      name: "全部勾选",
    });
    fireEvent.click(selectAllButton);
    fireEvent.click(await screen.findByRole("button", { name: "全部取消" }));
    fireEvent.click(await screen.findByRole("button", { name: "全部勾选" }));

    await waitFor(() => {
      const checkboxes = screen.getAllByRole("checkbox");
      expect(checkboxes).toHaveLength(8);
      checkboxes.forEach((checkbox) => {
        expect(checkbox).toHaveAttribute("data-state", "checked");
      });
    });
  });

  it("uses the persisted install root before running detection", async () => {
    toolsApiMock.getInstallRoot.mockResolvedValue("E:\\CustomTools");

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Wizard onComplete={vi.fn()} />
      </QueryClientProvider>,
    );

    expect(await screen.findByDisplayValue("E:\\CustomTools")).toBeInTheDocument();

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "E:\\CustomTools",
        false,
      );
    });
  });

  it("forces a fresh detect pass when the post-uninstall refresh token changes", async () => {
    toolsApiMock.detectTools
      .mockResolvedValueOnce(buildDetectResults(["claude-code-cli"]))
      .mockResolvedValueOnce(buildDetectResults([]));

    const queryClient = createTestQueryClient();
    const { rerender } = render(
      <QueryClientProvider client={queryClient}>
        <Wizard onComplete={vi.fn()} forceDetectionRefreshToken={0} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        false,
      );
    });
    await waitFor(() => {
      expect(screen.getAllByRole("checkbox")).toHaveLength(8);
    });

    rerender(
      <QueryClientProvider client={queryClient}>
        <Wizard onComplete={vi.fn()} forceDetectionRefreshToken={1} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenLastCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        true,
      );
    });
    await waitFor(() => {
      expect(screen.getAllByRole("checkbox")).toHaveLength(9);
    });
  });

  it("shows the most recent retained session in install progress when no step is active", async () => {
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));
    toolsApiMock.resolveInstallPlan.mockResolvedValue({
      steps: [
        {
          toolId: "codex-cli",
          toolName: "Codex (CLI)",
          category: "tool",
          reason: "selected",
          isInstalled: false,
        },
        {
          toolId: "gemini-cli",
          toolName: "Gemini CLI",
          category: "tool",
          reason: "selected",
          isInstalled: false,
        },
      ],
    });
    toolsApiMock.executeInstallPlan.mockResolvedValue(undefined);

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Wizard onComplete={vi.fn()} initialSelectedToolIds={["codex-cli", "gemini-cli"]} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        false,
      );
    });

    fireEvent.click(
      await screen.findByRole("button", { name: /开始安装|寮€濮嬪畨瑁?/ }),
    );

    await waitFor(() => {
      expect(toolsApiMock.executeInstallPlan).toHaveBeenCalledTimes(1);
    });

    act(() => {
      installProgressListener?.({
        toolId: "codex-cli",
        toolName: "Codex (CLI)",
        phase: "complete",
        percent: 100,
        message: "Codex complete",
      });
      installProgressListener?.({
        toolId: "gemini-cli",
        toolName: "Gemini CLI",
        phase: "complete",
        percent: 100,
        message: "Gemini complete",
      });

      installLogListener?.({
        toolId: "codex-cli",
        toolName: "Codex (CLI)",
        sessionId: "session-1",
        timestamp: "2026-05-09T12:00:00.000Z",
        level: "success",
        kind: "result",
        line: "Codex retained session",
        exitCode: 0,
      });
      installLogListener?.({
        toolId: "gemini-cli",
        toolName: "Gemini CLI",
        sessionId: "session-2",
        timestamp: "2026-05-09T12:01:00.000Z",
        level: "success",
        kind: "result",
        line: "Gemini retained session",
        exitCode: 0,
      });
    });

    expect(
      await screen.findAllByText("Gemini retained session"),
    ).not.toHaveLength(0);
    expect(screen.queryByText("Codex retained session")).not.toBeInTheDocument();
  });

  it("shows the active tool message plus recent retained activity in the current action area", async () => {
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));
    toolsApiMock.resolveInstallPlan.mockResolvedValue({
      steps: [
        {
          toolId: "codex-cli",
          toolName: "Codex (CLI)",
          category: "tool",
          reason: "selected",
          isInstalled: false,
        },
      ],
    });
    toolsApiMock.executeInstallPlan.mockResolvedValue(undefined);

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Wizard onComplete={vi.fn()} initialSelectedToolIds={["codex-cli"]} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        false,
      );
    });

    fireEvent.click(
      await screen.findByRole("button", { name: /开始安装/ }),
    );

    await waitFor(() => {
      expect(toolsApiMock.executeInstallPlan).toHaveBeenCalledTimes(1);
    });

    act(() => {
      installProgressListener?.({
        toolId: "codex-cli",
        toolName: "Codex (CLI)",
        phase: "installing",
        percent: 42,
        message: "Working...",
      });

      installLogListener?.({
        toolId: "codex-cli",
        toolName: "Codex (CLI)",
        sessionId: "session-active",
        timestamp: "2026-05-09T12:00:00.000Z",
        level: "info",
        kind: "phase",
        line: "Preparing managed runtime",
      });
      installLogListener?.({
        toolId: "codex-cli",
        toolName: "Codex (CLI)",
        sessionId: "session-active",
        timestamp: "2026-05-09T12:00:05.000Z",
        level: "info",
        kind: "command",
        line: "npm install -g @openai/codex",
      });
      installLogListener?.({
        toolId: "codex-cli",
        toolName: "Codex (CLI)",
        sessionId: "session-active",
        timestamp: "2026-05-09T12:00:08.000Z",
        level: "stdout",
        kind: "output",
        line: "Created shim at D:\\AgenticTools\\codex\\codex.cmd",
      });
      installLogListener?.({
        toolId: "codex-cli",
        toolName: "Codex (CLI)",
        sessionId: "session-active",
        timestamp: "2026-05-09T12:00:10.000Z",
        level: "stdout",
        kind: "output",
        line: "Finalizing install",
      });
    });

    const currentAction = await screen.findByLabelText("当前操作");
    expect(within(currentAction).getByText("当前操作")).toBeInTheDocument();
    expect(within(currentAction).getByText("Codex (CLI)")).toBeInTheDocument();
    expect(
      within(currentAction).getByText("Finalizing install"),
    ).toBeInTheDocument();
    expect(
      within(currentAction).getByText("npm install -g @openai/codex"),
    ).toBeInTheDocument();
    expect(
      within(currentAction).getByText("Created shim at D:\\AgenticTools\\codex\\codex.cmd"),
    ).toBeInTheDocument();
    expect(
      within(currentAction).queryByText("Preparing managed runtime"),
    ).not.toBeInTheDocument();
    expect(within(currentAction).queryByText("Working...")).not.toBeInTheDocument();
    expect(screen.getByText("总进度")).toBeInTheDocument();
    expect(screen.getByText("0 / 1 个工具")).toBeInTheDocument();
  });

  it("does not show the completion state before any install progress event arrives", async () => {
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));
    toolsApiMock.resolveInstallPlan.mockResolvedValue({
      steps: [
        {
          toolId: "codex-cli",
          toolName: "Codex (CLI)",
          category: "tool",
          reason: "selected",
          isInstalled: false,
        },
      ],
    });
    toolsApiMock.executeInstallPlan.mockResolvedValue(undefined);

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Wizard onComplete={vi.fn()} initialSelectedToolIds={["codex-cli"]} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        false,
      );
    });

    fireEvent.click(
      await screen.findByRole("button", { name: /开始安装|寮€濮嬪畨瑁?/ }),
    );

    await waitFor(() => {
      expect(toolsApiMock.executeInstallPlan).toHaveBeenCalledTimes(1);
    });

    expect(screen.getByText("Codex (CLI)")).toBeInTheDocument();
    expect(screen.queryByText(/安装完成|瀹夎瀹屾垚/)).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /进入管理|杩涘叆绠＄悊/ }),
    ).not.toBeInTheDocument();
  });
  it("switches to the install progress view immediately after clicking start", async () => {
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));
    const deferredPlan = createDeferred<{
      steps: Array<{
        toolId: string;
        toolName: string;
        category: string;
        reason: string;
        isInstalled: boolean;
      }>;
    }>();
    toolsApiMock.resolveInstallPlan.mockReturnValue(deferredPlan.promise);

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Wizard onComplete={vi.fn()} initialSelectedToolIds={["codex-cli"]} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        false,
      );
    });

    fireEvent.click(
      (await screen.findByText(/开始安装/)).closest("button") as HTMLElement,
    );

    expect(await screen.findByText(/瀹夎涓?|安装中/)).toBeInTheDocument();
    expect(screen.getByText("Codex (CLI)")).toBeInTheDocument();
    expect(toolsApiMock.executeInstallPlan).not.toHaveBeenCalled();
  });

  it("uses cached detection after loading a saved install root", async () => {
    const savedRoot = "E:\\SavedTools";
    toolsApiMock.getInstallRoot.mockResolvedValue(savedRoot);

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Wizard onComplete={vi.fn()} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        savedRoot,
        false,
      );
    });
  });

  it("forces a fresh detect pass only when clicking refresh", async () => {
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Wizard onComplete={vi.fn()} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        false,
      );
    });

    fireEvent.click(screen.getByRole("button", { name: "重新检测" }));

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenLastCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        true,
      );
    });
  });

  it("shows a visible error state and retry action when tool catalog loading fails", async () => {
    const user = userEvent.setup();
    toolsApiMock.getToolCatalog.mockRejectedValueOnce(new Error("catalog unavailable"));

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Wizard onComplete={vi.fn()} />
      </QueryClientProvider>,
    );

    expect(await screen.findByText("Failed to load tool catalog.")).toBeInTheDocument();
    expect(screen.getByText("Catalog is required before detection and installation can continue.")).toBeInTheDocument();
    expect(toolsApiMock.detectTools).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "Retry" }));

    await waitFor(() => {
      expect(toolsApiMock.getToolCatalog).toHaveBeenCalledTimes(2);
    });
  });

  it("filters out tools that are not installable on the current platform", async () => {
    toolsApiMock.getToolCatalog.mockResolvedValue([
      buildCatalogItem("codex-cli"),
      buildCatalogItem("gemini-cli", {
        platformSupport: {
          windows: "planned",
          macos: "implemented",
          linux: "implemented",
        },
        capabilities: {
          ...buildCatalogItem("gemini-cli").capabilities,
          canInstall: false,
          canUninstall: false,
          canLaunch: false,
        },
      }),
    ]);
    toolsApiMock.detectTools.mockResolvedValue([
      {
        installed: false,
        version: undefined,
        installPath: undefined,
      },
    ]);

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Wizard onComplete={vi.fn()} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        ["codex-cli"],
        "D:\\AgenticTools",
        false,
      );
    });

    expect(await screen.findByText("Codex (CLI)")).toBeInTheDocument();
    expect(screen.queryByText("Gemini CLI")).not.toBeInTheDocument();
  });
});
