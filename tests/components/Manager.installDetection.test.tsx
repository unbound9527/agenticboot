import { QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Manager } from "@/pages/Manager";
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

function createDeferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe("Manager install detection", () => {
  beforeEach(() => {
    toolsApiMock.getInstalledTools.mockResolvedValue([]);
    toolsApiMock.getInstallRoot.mockResolvedValue("D:\\AgenticTools");
    toolsApiMock.checkToolUpdates.mockResolvedValue([]);
    toolsApiMock.uninstallTool.mockResolvedValue(undefined);
    toolsApiMock.detectTools.mockResolvedValue(
      buildDetectResults(["opencode-cli"]),
    );
    toolsApiMock.resolveInstallPlan.mockReset();
    toolsApiMock.executeInstallPlan.mockReset();
    toolsApiMock.setInstallRoot.mockReset();
    installProgressListener = null;
    toolsApiMock.onInstallProgress.mockImplementation(async (callback) => {
      installProgressListener = callback;
      return () => {
        installProgressListener = null;
      };
    });
    toolsApiMock.onInstallLog.mockImplementation(async (_callback) => {
      installLogListener = _callback;
      return () => {
        installLogListener = null;
      };
    });
  });

  it("shows externally detected tools as installed with an uninstall button", async () => {
    const user = userEvent.setup();
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        false,
      );
    });

    const openCodeCard = (await screen.findByText("OpenCode (CLI)")).closest(
      ".claude-card",
    );
    expect(openCodeCard).not.toBeNull();
    const uninstallButton = within(openCodeCard as HTMLElement).getByRole("button");
    await user.click(uninstallButton);

    await waitFor(() => {
      expect(toolsApiMock.uninstallTool).toHaveBeenCalledWith(
        "opencode-cli",
        "D:\\AgenticTools",
      );
    });
  });
  it("uninstalls managed tools using the tool's recorded install root", async () => {
    const user = userEvent.setup();
    toolsApiMock.getInstalledTools.mockResolvedValue([
      {
        id: "codex-cli",
        name: "Codex (CLI)",
        version: "0.1.0",
        installPath: "C:\\ManagedRoot\\codex-cli",
        installRoot: "C:\\ManagedRoot",
        category: "tool",
        status: "installed",
      },
    ]);
    toolsApiMock.getInstallRoot.mockResolvedValue("D:\\DifferentRoot");
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    const codexCard = (await screen.findByText("Codex (CLI)")).closest(
      ".claude-card",
    );
    expect(codexCard).not.toBeNull();
    const uninstallButton = within(codexCard as HTMLElement).getByRole("button");
    await user.click(uninstallButton);

    await waitFor(() => {
      expect(toolsApiMock.uninstallTool).toHaveBeenCalledWith(
        "codex-cli",
        "C:\\ManagedRoot",
      );
    });
  });

  it("moves an uninstalled managed tool back to the available list", async () => {
    const user = userEvent.setup();
    toolsApiMock.getInstalledTools
      .mockResolvedValueOnce([
        {
          id: "codex-cli",
          name: "Codex (CLI)",
          version: "0.1.0",
          installPath: "D:\\AgenticTools\\codex-cli",
          installRoot: "D:\\AgenticTools",
          category: "tool",
          status: "installed",
        },
      ])
      .mockResolvedValue([]);
    toolsApiMock.detectTools
      .mockResolvedValueOnce(buildDetectResults(["codex-cli"]))
      .mockResolvedValueOnce(buildDetectResults([]));

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    const codexCard = (await screen.findByText("Codex (CLI)")).closest(
      ".claude-card",
    );
    expect(codexCard).not.toBeNull();

    await user.click(within(codexCard as HTMLElement).getByRole("button"));

    await waitFor(() => {
      expect(toolsApiMock.uninstallTool).toHaveBeenCalledWith(
        "codex-cli",
        "D:\\AgenticTools",
      );
    });
    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenLastCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        true,
      );
    });
    await waitFor(() => {
      expect(screen.getAllByRole("tab")[0]).toHaveTextContent("(0)");
      expect(screen.getAllByRole("tab")[1]).toHaveTextContent("(9)");
    });
  });

  it("does not treat errored database records as installed tools", async () => {
    toolsApiMock.getInstalledTools.mockResolvedValue([
      {
        id: "openclaw",
        name: "OpenClaw",
        version: null,
        installPath: "D:\\AgenticTools\\openclaw",
        installRoot: "D:\\AgenticTools",
        category: "tool",
        status: "error",
      },
    ]);
    toolsApiMock.getInstallRoot.mockResolvedValue("D:\\AgenticTools");
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        false,
      );
    });

    expect(screen.queryByText("OpenClaw")).not.toBeInTheDocument();
  });

  it("installs a single available tool directly instead of routing back to the wizard", async () => {
    const user = userEvent.setup();
    const onInstallMore = vi.fn();
    const plan = {
      steps: [
        {
          toolId: "codex-cli",
          toolName: "Codex (CLI)",
          category: "tool",
          reason: "selected",
          isInstalled: false,
        },
      ],
    };
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));
    toolsApiMock.resolveInstallPlan.mockResolvedValue(plan);
    toolsApiMock.executeInstallPlan.mockResolvedValue(undefined);

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager onInstallMore={onInstallMore} />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        false,
      );
    });

    await user.click(screen.getAllByRole("tab")[1]);

    const codexCard = (await screen.findByText("Codex (CLI)")).closest(
      ".claude-card",
    );
    expect(codexCard).not.toBeNull();
    await user.click(within(codexCard as HTMLElement).getByRole("button"));

    await waitFor(() => {
      expect(toolsApiMock.resolveInstallPlan).toHaveBeenCalledWith(
        ["codex-cli"],
        "D:\\AgenticTools",
      );
    });
    await waitFor(() => {
      expect(toolsApiMock.executeInstallPlan).toHaveBeenCalledWith(
        plan,
        "D:\\AgenticTools",
      );
    });
    expect(onInstallMore).not.toHaveBeenCalled();
  });

  it("shows immediate install feedback before the install plan finishes resolving", async () => {
    const user = userEvent.setup();
    const deferredPlan = createDeferred<{
      steps: Array<{
        toolId: string;
        toolName: string;
        category: string;
        reason: string;
        isInstalled: boolean;
      }>;
    }>();
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));
    toolsApiMock.resolveInstallPlan.mockReturnValue(deferredPlan.promise);

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        false,
      );
    });

    await user.click(screen.getAllByRole("tab")[1]);

    const codexCard = (await screen.findByText("Codex (CLI)")).closest(
      ".claude-card",
    );
    expect(codexCard).not.toBeNull();

    await user.click(
      within(codexCard as HTMLElement).getByRole("button", { name: /安装/ }),
    );

    await waitFor(() => {
      expect(toolsApiMock.resolveInstallPlan).toHaveBeenCalledWith(
        ["codex-cli"],
        "D:\\AgenticTools",
      );
    });
    expect(
      within(codexCard as HTMLElement).queryByRole("button", { name: /安装/ }),
    ).not.toBeInTheDocument();
    expect(
      within(codexCard as HTMLElement).getByText(/Preparing installation|准备安装/),
    ).toBeInTheDocument();
    expect(toolsApiMock.executeInstallPlan).not.toHaveBeenCalled();
  });

  it("uses the updated install root for detection and direct installs", async () => {
    const user = userEvent.setup();
    const plan = {
      steps: [
        {
          toolId: "codex-cli",
          toolName: "Codex (CLI)",
          category: "tool",
          reason: "selected",
          isInstalled: false,
        },
      ],
    };
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));
    toolsApiMock.resolveInstallPlan.mockResolvedValue(plan);
    toolsApiMock.executeInstallPlan.mockResolvedValue(undefined);
    toolsApiMock.setInstallRoot.mockResolvedValue(undefined);

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    const rootInput = await screen.findByDisplayValue("D:\\AgenticTools");
    fireEvent.change(rootInput, {
      target: { value: "E:\\CustomTools" },
    });
    fireEvent.blur(rootInput);

    await waitFor(() => {
      expect(toolsApiMock.setInstallRoot).toHaveBeenCalledWith(
        "E:\\CustomTools",
      );
    });
    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenLastCalledWith(
        [...TOOL_IDS],
        "E:\\CustomTools",
        false,
      );
    });

    await user.click(screen.getAllByRole("tab")[1]);

    const codexCard = (await screen.findByText("Codex (CLI)")).closest(
      ".claude-card",
    );
    expect(codexCard).not.toBeNull();
    await user.click(within(codexCard as HTMLElement).getByRole("button"));

    await waitFor(() => {
      expect(toolsApiMock.resolveInstallPlan).toHaveBeenCalledWith(
        ["codex-cli"],
        "E:\\CustomTools",
      );
    });
    await waitFor(() => {
      expect(toolsApiMock.executeInstallPlan).toHaveBeenCalledWith(
        plan,
        "E:\\CustomTools",
      );
    });
  });

  it("uses cached detection on initial mount", async () => {
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        false,
      );
    });
  });

  it("forces a fresh detect pass only when clicking refresh", async () => {
    const user = userEvent.setup();

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        false,
      );
    });

    await user.click(
      screen.getByRole("button", { name: "重新检测" }),
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenLastCalledWith(
        [...TOOL_IDS],
        "D:\\AgenticTools",
        true,
      );
    });
  });

  it("shows in-flight install messages for available tools", async () => {
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    await userEvent.click(screen.getAllByRole("tab")[1]);
    expect(await screen.findByText("OpenClaw")).toBeInTheDocument();

    await act(async () => {
      installProgressListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        phase: "configuring",
        percent: 65,
        message: "Waiting for the official OpenClaw installer to finish...",
      });
    });

    expect(
      await screen.findByText(
        "Waiting for the official OpenClaw installer to finish...",
      ),
    ).toBeInTheDocument();
  });

  it("shows recent activity lines in the console without requiring the raw tab", async () => {
    const user = userEvent.setup();
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    await user.click(screen.getAllByRole("tab")[1]);
    expect(await screen.findByText("OpenClaw")).toBeInTheDocument();

    await act(async () => {
      installLogListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        sessionId: "session-1",
        timestamp: "2026-05-12T09:00:00.000Z",
        level: "info",
        kind: "session-started",
        line: "Install session started",
      });
      installLogListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        sessionId: "session-1",
        timestamp: "2026-05-12T09:00:01.000Z",
        level: "info",
        kind: "command",
        line: "Running installer command",
        command: "installer.exe /S",
      });
      installLogListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        sessionId: "session-1",
        timestamp: "2026-05-12T09:00:02.000Z",
        level: "stdout",
        kind: "output",
        line: "Download complete: installer payload ready",
      });
    });

    const openClawCard = (await screen.findByText("OpenClaw")).closest(".claude-card");
    expect(openClawCard).not.toBeNull();

    await user.click(
      within(openClawCard as HTMLElement).getByRole("button", { name: /Console/i }),
    );

    expect(await screen.findByText("最近活动")).toBeInTheDocument();
    expect(screen.getAllByText("Running installer command").length).toBeGreaterThan(1);
    expect(
      screen.getByRole("tab", { name: "原始输出" }),
    ).toBeInTheDocument();
  });

  it("keeps retained session review available with summary and raw output after completion", async () => {
    const user = userEvent.setup();
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    await user.click(screen.getAllByRole("tab")[1]);
    expect(await screen.findByText("Codex (CLI)")).toBeInTheDocument();

    await act(async () => {
      installLogListener?.({
        toolId: "codex-cli",
        toolName: "Codex (CLI)",
        sessionId: "session-2",
        timestamp: "2026-05-12T09:10:00.000Z",
        level: "info",
        kind: "session-started",
        line: "Install session started",
      });
      installLogListener?.({
        toolId: "codex-cli",
        toolName: "Codex (CLI)",
        sessionId: "session-2",
        timestamp: "2026-05-12T09:10:01.000Z",
        level: "stdout",
        kind: "output",
        line: "Version 1.2.3 installed successfully",
      });
      installLogListener?.({
        toolId: "codex-cli",
        toolName: "Codex (CLI)",
        sessionId: "session-2",
        timestamp: "2026-05-12T09:10:02.000Z",
        level: "success",
        kind: "result",
        line: "Install completed",
        exitCode: 0,
      });
    });

    const codexCard = (await screen.findByText("Codex (CLI)")).closest(".claude-card");
    expect(codexCard).not.toBeNull();

    await user.click(
      within(codexCard as HTMLElement).getByRole("button", { name: /Console/i }),
    );

    expect(await screen.findByText("最近活动")).toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "原始输出" }));

    await waitFor(() => {
      expect(
        screen.getAllByText("Version 1.2.3 installed successfully").length,
      ).toBeGreaterThan(1);
    });
    expect(screen.getAllByText("Install completed").length).toBeGreaterThan(0);
  });

  it("falls back to retained summary text in the header when activity is empty", async () => {
    const user = userEvent.setup();
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    await user.click(screen.getAllByRole("tab")[1]);
    expect(await screen.findByText("OpenClaw")).toBeInTheDocument();

    await act(async () => {
      installLogListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        sessionId: "session-empty-activity",
        timestamp: "2026-05-12T09:20:00.000Z",
        level: "info",
        kind: "session-started",
        line: "Install session started",
      });
    });

    const openClawCard = (await screen.findByText("OpenClaw")).closest(".claude-card");
    expect(openClawCard).not.toBeNull();

    await user.click(
      within(openClawCard as HTMLElement).getByRole("button", { name: /Console/i }),
    );

    const consoleHeader = screen.getByRole("button", {
      name: /OpenClaw.*Install session started/i,
    });
    expect(consoleHeader).toBeInTheDocument();
    expect(consoleHeader).not.toHaveTextContent("[session-started]");
    expect(screen.queryByText("最近活动")).not.toBeInTheDocument();
  });
  it("resets to the summary view and reopens the console when a new session replaces the current one", async () => {
    const user = userEvent.setup();
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    await user.click(screen.getAllByRole("tab")[1]);
    expect(await screen.findByText("OpenClaw")).toBeInTheDocument();

    await act(async () => {
      installLogListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        sessionId: "session-a",
        timestamp: "2026-05-12T10:00:00.000Z",
        level: "info",
        kind: "session-started",
        line: "First session started",
      });
      installLogListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        sessionId: "session-a",
        timestamp: "2026-05-12T10:00:01.000Z",
        level: "info",
        kind: "command",
        line: "First session command",
        command: "installer-a.exe /S",
      });
    });

    const openClawCard = (await screen.findByText("OpenClaw")).closest(".claude-card");
    expect(openClawCard).not.toBeNull();

    await user.click(
      within(openClawCard as HTMLElement).getByRole("button", { name: /Console/i }),
    );

    await user.click(screen.getByRole("tab", { name: /Raw Output|原始输出/ }));
    expect(
      screen.getByRole("tab", { name: /Raw Output|原始输出/, selected: true }),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: /OpenClaw.*First session command/i }),
    );
    expect(
      screen.queryByRole("tab", { name: /Summary|摘要/ }),
    ).not.toBeInTheDocument();

    await act(async () => {
      installLogListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        sessionId: "session-b",
        timestamp: "2026-05-12T10:01:00.000Z",
        level: "info",
        kind: "session-started",
        line: "Second session started",
      });
      installLogListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        sessionId: "session-b",
        timestamp: "2026-05-12T10:01:01.000Z",
        level: "info",
        kind: "command",
        line: "Second session command",
        command: "installer-b.exe /S",
      });
    });

    expect(
      await screen.findByRole("tab", { name: /Summary|摘要/, selected: true }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: /Raw Output|原始输出/, selected: false }),
    ).toBeInTheDocument();
    expect(screen.getAllByText("Second session command").length).toBeGreaterThan(1);
    expect(screen.queryByText("First session command")).not.toBeInTheDocument();
  });
});
