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

describe("Manager install detection", () => {
  beforeEach(() => {
    toolsApiMock.getInstalledTools.mockResolvedValue([]);
    toolsApiMock.getInstallRoot.mockResolvedValue("D:\\AITools");
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
  });

  it("shows externally detected tools as installed without offering uninstall", async () => {
    const { container } = render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AITools",
      );
    });

    expect(await screen.findByText("OpenCode (CLI)")).toBeInTheDocument();
    expect(container.querySelectorAll('button[title="鍗歌浇"]')).toHaveLength(
      0,
    );
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

    const { container } = render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    expect(await screen.findByText("Codex (CLI)")).toBeInTheDocument();

    const uninstallButton = container.querySelector('button[title="卸载"]');
    expect(uninstallButton).not.toBeNull();
    await user.click(uninstallButton as HTMLElement);

    await waitFor(() => {
      expect(toolsApiMock.uninstallTool).toHaveBeenCalledWith(
        "codex-cli",
        "C:\\ManagedRoot",
      );
    });
  });

  it("does not treat errored database records as installed tools", async () => {
    toolsApiMock.getInstalledTools.mockResolvedValue([
      {
        id: "openclaw",
        name: "OpenClaw",
        version: null,
        installPath: "D:\\AITools\\openclaw",
        installRoot: "D:\\AITools",
        category: "tool",
        status: "error",
      },
    ]);
    toolsApiMock.getInstallRoot.mockResolvedValue("D:\\AITools");
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));

    const { container } = render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        [...TOOL_IDS],
        "D:\\AITools",
      );
    });

    expect(screen.queryByText("OpenClaw")).not.toBeInTheDocument();
    expect(container.querySelectorAll('button[title="閸楁瓕娴?]')).toHaveLength(
      0,
    );
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
        "D:\\AITools",
      );
    });

    await user.click(screen.getByRole("tab", { name: /未安装/ }));

    const codexCard = (await screen.findByText("Codex (CLI)")).closest(
      ".claude-card",
    );
    expect(codexCard).not.toBeNull();
    await user.click(within(codexCard as HTMLElement).getByRole("button"));

    await waitFor(() => {
      expect(toolsApiMock.resolveInstallPlan).toHaveBeenCalledWith(
        ["codex-cli"],
        "D:\\AITools",
      );
    });
    await waitFor(() => {
      expect(toolsApiMock.executeInstallPlan).toHaveBeenCalledWith(
        plan,
        "D:\\AITools",
      );
    });
    expect(onInstallMore).not.toHaveBeenCalled();
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

    const rootInput = await screen.findByDisplayValue("D:\\AITools");
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
      );
    });

    await user.click(screen.getByRole("tab", { name: /未安装/ }));

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

  it("shows in-flight install messages for available tools", async () => {
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    await userEvent.click(screen.getByRole("tab", { name: /未安装|鏈畨瑁?/ }));
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
});
