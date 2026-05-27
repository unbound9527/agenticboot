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
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
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
  onResolveProgress: vi.fn(),
  onResolveComplete: vi.fn(),
  onInstallProgress: vi.fn(),
  onInstallLog: vi.fn(),
  onInstallComplete: vi.fn(),
  onInstallError: vi.fn(),
  openFolder: vi.fn(),
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
let resolveProgressListener:
  | ((progress: {
      toolId: string;
      toolName: string;
      phase: "resolving" | "resolved" | "error";
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

function buildDetectResultsWithInstallPaths(
  installPaths: Partial<Record<(typeof TOOL_IDS)[number], string>>,
) {
  return TOOL_IDS.map((id) => ({
    installed: Boolean(installPaths[id]),
    version: installPaths[id] ? "1.0.0" : undefined,
    installPath: installPaths[id],
  }));
}

function buildInstalledToolsRecords(
  records: Array<{
    id: (typeof TOOL_IDS)[number];
    installPath: string;
    installRoot?: string;
    status?: "installed" | "detected";
    stateSource?: "managed" | "external_detected";
  }>,
) {
  return records.map((record) => ({
    id: record.id,
    name: TOOL_NAMES[record.id],
    version: "1.0.0",
    installPath: record.installPath,
    installRoot: record.installRoot ?? "D:\\AgenticTools",
    category: "tool",
    status: record.status ?? "detected",
    stateSource: record.stateSource ?? "external_detected",
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

async function confirmUninstall(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole("button", { name: "卸载" }));
}

describe("Manager install detection", () => {
  beforeEach(() => {
    toolsApiMock.getToolCatalog.mockResolvedValue(buildToolCatalog());
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
    resolveProgressListener = null;
    toolsApiMock.onResolveProgress.mockImplementation(async (callback) => {
      resolveProgressListener = callback;
      return () => {
        resolveProgressListener = null;
      };
    });
    toolsApiMock.onResolveComplete.mockImplementation(async (_callback) => () => {});
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

  afterEach(() => {
    vi.useRealTimers();
  });

  it("shows externally detected tools as installed with an uninstall button", async () => {
    const user = userEvent.setup();
    toolsApiMock.getInstalledTools
      .mockResolvedValueOnce([])
      .mockResolvedValue(
        buildInstalledToolsRecords([
          {
            id: "opencode-cli",
            installPath: "C:\\Tools\\opencode-cli",
          },
        ]),
      );
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
    const uninstallButton = within(openCodeCard as HTMLElement).getByTitle("卸载");
    await user.click(uninstallButton);
    await confirmUninstall(user);

    await waitFor(() => {
      expect(toolsApiMock.uninstallTool).toHaveBeenCalledWith(
        "opencode-cli",
        "C:\\Tools\\opencode-cli",
      );
    });
  });

  it("shows cached externally detected tools before background detection finishes", async () => {
    const deferredDetect = createDeferred<ReturnType<typeof buildDetectResults>>();
    toolsApiMock.getInstalledTools.mockResolvedValue([
      {
        id: "opencode-cli",
        name: "OpenCode (CLI)",
        version: "1.0.0",
        installPath: "C:\\Users\\me\\AppData\\Roaming\\npm",
        installRoot: "D:\\AgenticTools",
        category: "tool",
        status: "detected",
      },
    ]);
    toolsApiMock.detectTools.mockReturnValue(deferredDetect.promise);

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    expect(await screen.findByText("OpenCode (CLI)")).toBeInTheDocument();
  });

  it("shows an uninstall button for externally detected Hermes", async () => {
    const user = userEvent.setup();
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults(["hermes"]));
    toolsApiMock.getInstalledTools
      .mockResolvedValueOnce([])
      .mockResolvedValue(
        buildInstalledToolsRecords([
          {
            id: "hermes",
            installPath: "C:\\Tools\\hermes",
          },
        ]),
      );

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

    const hermesCard = (await screen.findByText("Hermes (Web UI)")).closest(
      ".claude-card",
    );
    expect(hermesCard).not.toBeNull();

    await user.click(within(hermesCard as HTMLElement).getByTitle("卸载"));
    await confirmUninstall(user);

    await waitFor(() => {
      expect(toolsApiMock.uninstallTool).toHaveBeenCalledWith(
        "hermes",
        "C:\\Tools\\hermes",
      );
    });
  });

  it("shows an uninstall button for detected Hermes inside the install root", async () => {
    const user = userEvent.setup();
    toolsApiMock.detectTools.mockResolvedValue(
      buildDetectResultsWithInstallPaths({
        hermes: "D:\\AgenticTools\\hermes",
      }),
    );
    toolsApiMock.getInstalledTools
      .mockResolvedValueOnce([])
      .mockResolvedValue(
        buildInstalledToolsRecords([
          {
            id: "hermes",
            installPath: "D:\\AgenticTools\\hermes",
            installRoot: "D:\\AgenticTools",
            stateSource: "managed",
          },
        ]),
      );

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

    const hermesCard = (await screen.findByText("Hermes (Web UI)")).closest(
      ".claude-card",
    );
    expect(hermesCard).not.toBeNull();

    await user.click(within(hermesCard as HTMLElement).getByTitle("卸载"));
    await confirmUninstall(user);

    await waitFor(() => {
      expect(toolsApiMock.uninstallTool).toHaveBeenCalledWith(
        "hermes",
        "D:\\AgenticTools",
      );
    });
  });

  it("uninstalls externally detected CLI tools using the detected install path", async () => {
    const user = userEvent.setup();
    toolsApiMock.detectTools.mockResolvedValue(
      buildDetectResultsWithInstallPaths({
        "opencode-cli": "C:\\Users\\me\\AppData\\Roaming\\npm",
      }),
    );
    toolsApiMock.getInstalledTools
      .mockResolvedValueOnce([])
      .mockResolvedValue(
        buildInstalledToolsRecords([
          {
            id: "opencode-cli",
            installPath: "C:\\Users\\me\\AppData\\Roaming\\npm",
          },
        ]),
      );

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

    await user.click(within(openCodeCard as HTMLElement).getByTitle("卸载"));
    await confirmUninstall(user);

    await waitFor(() => {
      expect(toolsApiMock.uninstallTool).toHaveBeenCalledWith(
        "opencode-cli",
        "C:\\Users\\me\\AppData\\Roaming\\npm",
      );
    });
  });

  it("shows uninstall for detected desktop tools even when no install path is available", async () => {
    const user = userEvent.setup();
    toolsApiMock.detectTools.mockResolvedValue(
      TOOL_IDS.map((id) =>
        id === "codex-desktop"
          ? {
              installed: true,
              version: "1.0.0",
              installPath: undefined,
            }
          : {
              installed: false,
              version: undefined,
              installPath: undefined,
            },
        ),
    );
    toolsApiMock.getInstalledTools
      .mockResolvedValueOnce([])
      .mockResolvedValue(
        buildInstalledToolsRecords([
          {
            id: "codex-desktop",
            installPath: "",
          },
        ]),
      );

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

    const codexDesktopCard = (
      await screen.findAllByText("Codex (桌面版)")
    )[0].closest(".claude-card");
    expect(codexDesktopCard).not.toBeNull();

    const uninstallButton = within(codexDesktopCard as HTMLElement).getByTitle("卸载");
    await user.click(uninstallButton);
    await confirmUninstall(user);

    await waitFor(() => {
      expect(toolsApiMock.uninstallTool).toHaveBeenCalledWith(
        "codex-desktop",
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
    const uninstallButton = within(codexCard as HTMLElement).getByTitle("卸载");
    await user.click(uninstallButton);
    await confirmUninstall(user);

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

    await user.click(within(codexCard as HTMLElement).getByTitle("卸载"));
    await confirmUninstall(user);

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

  it("keeps independent uninstall loading states for parallel uninstalls", async () => {
    const user = userEvent.setup();
    const firstUninstall = createDeferred<void>();
    const secondUninstall = createDeferred<void>();

    toolsApiMock.detectTools.mockResolvedValue(
      buildDetectResults(["codex-cli", "gemini-cli"]),
    );
    toolsApiMock.getInstalledTools
      .mockResolvedValueOnce([])
      .mockResolvedValue(
        buildInstalledToolsRecords([
          {
            id: "codex-cli",
            installPath: "C:\\Tools\\codex-cli",
          },
          {
            id: "gemini-cli",
            installPath: "C:\\Tools\\gemini-cli",
          },
        ]),
      );
    toolsApiMock.uninstallTool
      .mockReturnValueOnce(firstUninstall.promise)
      .mockReturnValueOnce(secondUninstall.promise);

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    const codexCard = (await screen.findByText("Codex (CLI)")).closest(
      ".claude-card",
    );
    const geminiCard = (await screen.findByText("Gemini CLI")).closest(
      ".claude-card",
    );
    expect(codexCard).not.toBeNull();
    expect(geminiCard).not.toBeNull();

    await user.click(within(codexCard as HTMLElement).getByTitle("卸载"));
    await confirmUninstall(user);
    await user.click(within(geminiCard as HTMLElement).getByTitle("卸载"));
    await confirmUninstall(user);

    await waitFor(() => {
      expect(toolsApiMock.uninstallTool).toHaveBeenNthCalledWith(
        1,
        "codex-cli",
        "C:\\Tools\\codex-cli",
      );
      expect(toolsApiMock.uninstallTool).toHaveBeenNthCalledWith(
        2,
        "gemini-cli",
        "C:\\Tools\\gemini-cli",
      );
    });

    expect(
      within(codexCard as HTMLElement).getByTitle("卸载"),
    ).toBeDisabled();
    expect(
      within(geminiCard as HTMLElement).getByTitle("卸载"),
    ).toBeDisabled();

    firstUninstall.resolve();
    await waitFor(() => {
      expect(
        within(codexCard as HTMLElement).getByTitle("卸载"),
      ).not.toBeDisabled();
    });
    expect(
      within(geminiCard as HTMLElement).getByTitle("卸载"),
    ).toBeDisabled();

    secondUninstall.resolve();
    await waitFor(() => {
      expect(
        within(geminiCard as HTMLElement).getByTitle("卸载"),
      ).not.toBeDisabled();
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

  it("surfaces resolve-progress updates on the tool card while plan resolution is in flight", async () => {
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

    await user.click(within(codexCard as HTMLElement).getByTitle("安装"));

    await act(async () => {
      resolveProgressListener?.({
        toolId: "codex-cli",
        toolName: "Codex (CLI)",
        phase: "resolving",
        message: "Checking dependency graph...",
      });
    });

    expect(
      within(codexCard as HTMLElement).getByText("Checking dependency graph..."),
    ).toBeInTheDocument();
  });

  it("opens the console immediately after clicking install, even before plan resolution", async () => {
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

    await user.click(screen.getAllByRole("tab")[1]);
    const codexCard = (await screen.findByText("Codex (CLI)")).closest(
      ".claude-card",
    );
    expect(codexCard).not.toBeNull();

    await user.click(
      within(codexCard as HTMLElement).getByTitle("安装"),
    );

    expect(
      await screen.findByText("System: Install requested."),
    ).toBeInTheDocument();
    expect(
      screen.getByText("System: Resolving install plan..."),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Codex \(CLI\)/i }),
    ).toBeInTheDocument();
    expect(toolsApiMock.executeInstallPlan).not.toHaveBeenCalled();
  });

  it("keeps console content when reopening and appends new output incrementally", async () => {
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

    await user.click(screen.getAllByRole("tab")[1]);
    const codexCard = (await screen.findByText("Codex (CLI)")).closest(
      ".claude-card",
    );
    expect(codexCard).not.toBeNull();

    await user.click(
      within(codexCard as HTMLElement).getByTitle("安装"),
    );

    expect(
      await screen.findByText("System: Resolving install plan..."),
    ).toBeInTheDocument();

    await user.click(within(codexCard as HTMLElement).getByTitle("控制台"));
    expect(
      screen.queryByText("System: Resolving install plan..."),
    ).not.toBeInTheDocument();

    await act(async () => {
      installLogListener?.({
        toolId: "codex-cli",
        toolName: "Codex (CLI)",
        sessionId: "session-reopen",
        timestamp: "2026-05-12T08:00:00.000Z",
        level: "info",
        kind: "session-started",
        line: "Install session started",
      });
      installLogListener?.({
        toolId: "codex-cli",
        toolName: "Codex (CLI)",
        sessionId: "session-reopen",
        timestamp: "2026-05-12T08:00:01.000Z",
        level: "info",
        kind: "command",
        line: "Running installer command",
        command: "installer.exe /S",
      });
    });

    await user.click(within(codexCard as HTMLElement).getByTitle("控制台"));
    expect(
      await screen.findByText("System: Resolving install plan..."),
    ).toBeInTheDocument();
    expect(screen.getByText("Running installer command")).toBeInTheDocument();
  });

  it("loads older console output when scrolling to the top", async () => {
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
        sessionId: "session-scroll",
        timestamp: "2026-05-12T11:00:00.000Z",
        level: "info",
        kind: "session-started",
        line: "Install session started",
      });

      for (let index = 1; index <= 120; index += 1) {
        installLogListener?.({
          toolId: "openclaw",
          toolName: "OpenClaw",
          sessionId: "session-scroll",
          timestamp: `2026-05-12T11:00:${String(index).padStart(2, "0")}.000Z`,
          level: "stdout",
          kind: "output",
          line: `Output line ${index}`,
        });
      }
    });

    const openClawCard = (await screen.findByText("OpenClaw")).closest(".claude-card");
    expect(openClawCard).not.toBeNull();

    await user.click(within(openClawCard as HTMLElement).getByTitle("控制台"));

    expect(screen.getByText("Output line 120")).toBeInTheDocument();
    expect(screen.queryByText("Output line 1")).not.toBeInTheDocument();

    fireEvent.scroll(screen.getByTestId("install-console-viewport"), {
      target: { scrollTop: 0 },
    });

    expect(await screen.findByText("Output line 1")).toBeInTheDocument();
  });

  it("shows immediate update feedback for installed tools before the update plan finishes resolving", async () => {
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

    toolsApiMock.getInstalledTools.mockResolvedValue([
      {
        id: "codex-cli",
        name: "Codex (CLI)",
        version: "codex 0.24.0",
        installPath: "D:\\AgenticTools\\codex-cli",
        installRoot: "D:\\AgenticTools",
        category: "tool",
        status: "installed",
      },
    ]);
    toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));
    toolsApiMock.checkToolUpdates.mockResolvedValue([
      {
        toolId: "codex-cli",
        currentVersion: "codex 0.24.0",
        latestVersion: "0.25.0",
      },
    ]);
    toolsApiMock.resolveInstallPlan.mockReturnValue(deferredPlan.promise);

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    const codexCard = (await screen.findByText("Codex (CLI)")).closest(
      ".claude-card",
    );
    expect(codexCard).not.toBeNull();

    await user.click(within(codexCard as HTMLElement).getByTitle("更新"));

    await waitFor(() => {
      expect(toolsApiMock.resolveInstallPlan).toHaveBeenCalledWith(
        ["codex-cli"],
        "D:\\AgenticTools",
      );
    });
    expect(
      within(codexCard as HTMLElement).getByText(/Preparing installation|鍑嗗瀹夎/),
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

  it("shows raw install output directly in the console", async () => {
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
      within(openClawCard as HTMLElement).getByRole("button", { name: /控制台/i }),
    );

    expect(
      screen.queryByRole("tab", { name: /Summary/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("tab", { name: /Raw Output/ }),
    ).not.toBeInTheDocument();
    expect(screen.getByText("Running installer command")).toBeInTheDocument();
    expect(
      screen.getByText("Download complete: installer payload ready"),
    ).toBeInTheDocument();
  });

  it("adds heartbeat-style system updates while a session is still running", async () => {
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
        sessionId: "session-heartbeat",
        timestamp: "2026-05-12T09:30:00.000Z",
        level: "info",
        kind: "session-started",
        line: "Install session started",
      });
    });

    const openClawCard = (await screen.findByText("OpenClaw")).closest(".claude-card");
    expect(openClawCard).not.toBeNull();

    await user.click(
      within(openClawCard as HTMLElement).getByRole("button", { name: /控制台/i }),
    );

    expect(screen.getByText("System: Creating install session...")).toBeInTheDocument();

    await act(async () => {
      await new Promise((resolve) => window.setTimeout(resolve, 2600));
    });

    expect(
      screen.getByText("System: Verifying installer is still running..."),
    ).toBeInTheDocument();

    await act(async () => {
      await new Promise((resolve) => window.setTimeout(resolve, 2600));
    });

    expect(
      screen.getByText("System: Waiting for next installer output..."),
    ).toBeInTheDocument();
  }, 10000);

  it("shows denser system summaries from progress updates inside the console", async () => {
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
        sessionId: "session-progress-density",
        timestamp: "2026-05-12T09:40:00.000Z",
        level: "info",
        kind: "session-started",
        line: "Install session started",
      });
      installProgressListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        phase: "configuring",
        percent: 65,
        message: "Waiting for the official OpenClaw installer to finish...",
      });
    });

    const openClawCard = (await screen.findByText("OpenClaw")).closest(".claude-card");
    expect(openClawCard).not.toBeNull();

    await user.click(
      within(openClawCard as HTMLElement).getByRole("button", { name: /控制台/i }),
    );

    expect(
      screen.getByText("System: configuring 65% complete."),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "System: Waiting for the official OpenClaw installer to finish...",
      ),
    ).toBeInTheDocument();
  });

  it("removes the console and button after installation completes", async () => {
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
    });

    let codexCard = (await screen.findByText("Codex (CLI)")).closest(".claude-card");
    expect(codexCard).not.toBeNull();

    await user.click(
      within(codexCard as HTMLElement).getByRole("button", { name: /控制台/i }),
    );

    expect(screen.getByText("Version 1.2.3 installed successfully")).toBeInTheDocument();

    await act(async () => {
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

    await waitFor(() => {
      expect(screen.queryByText("Version 1.2.3 installed successfully")).not.toBeInTheDocument();
    });

    codexCard = (await screen.findByText("Codex (CLI)")).closest(".claude-card");
    expect(codexCard).not.toBeNull();
    expect(
      within(codexCard as HTMLElement).queryByRole("button", { name: /控制台/i }),
    ).not.toBeInTheDocument();
  });

  it("shows only tool name and status in the console header", async () => {
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
      within(openClawCard as HTMLElement).getByRole("button", { name: /控制台/i }),
    );

    const consoleHeader = screen.getByRole("button", { name: /OpenClaw/i });
    expect(consoleHeader).toBeInTheDocument();
    expect(consoleHeader).not.toHaveTextContent("Install session started");
    expect(consoleHeader).toHaveTextContent("running");
  });

  it("shows the console button again for a new running session after completion", async () => {
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

    let openClawCard = (await screen.findByText("OpenClaw")).closest(".claude-card");
    expect(openClawCard).not.toBeNull();

    await user.click(
      within(openClawCard as HTMLElement).getByRole("button", { name: /控制台/i }),
    );

    expect(screen.getByText("First session command")).toBeInTheDocument();

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

    expect(screen.getByText("Second session command")).toBeInTheDocument();
    expect(screen.queryByText("First session command")).not.toBeInTheDocument();

    await act(async () => {
      installLogListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        sessionId: "session-b",
        timestamp: "2026-05-12T10:01:02.000Z",
        level: "success",
        kind: "result",
        line: "Second session finished",
        exitCode: 0,
      });
    });

    await waitFor(() => {
      expect(screen.queryByText("Second session command")).not.toBeInTheDocument();
    });

    await act(async () => {
      installLogListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        sessionId: "session-c",
        timestamp: "2026-05-12T10:02:00.000Z",
        level: "info",
        kind: "session-started",
        line: "Third session started",
      });
      installLogListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        sessionId: "session-c",
        timestamp: "2026-05-12T10:02:01.000Z",
        level: "info",
        kind: "command",
        line: "Third session command",
        command: "installer-c.exe /S",
      });
    });

    openClawCard = screen.getAllByText("OpenClaw")[0]?.closest(".claude-card") ?? null;
    expect(openClawCard).not.toBeNull();
    expect(
      within(openClawCard as HTMLElement).getByRole("button", { name: /控制台/i }),
    ).toBeInTheDocument();
  });
  it("keeps the console visible for errored sessions", async () => {
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
        sessionId: "session-error-visible",
        timestamp: "2026-05-12T10:10:00.000Z",
        level: "info",
        kind: "session-started",
        line: "Install session started",
      });
      installLogListener?.({
        toolId: "openclaw",
        toolName: "OpenClaw",
        sessionId: "session-error-visible",
        timestamp: "2026-05-12T10:10:01.000Z",
        level: "error",
        kind: "result",
        line: "Installer failed with exit code 1",
        exitCode: 1,
      });
    });

    const openClawCard = (await screen.findByText("OpenClaw")).closest(".claude-card");
    expect(openClawCard).not.toBeNull();

    await user.click(within(openClawCard as HTMLElement).getByTitle("控制台"));

    expect(
      screen.getByText("Installer failed with exit code 1"),
    ).toBeInTheDocument();
  });

  it("shows a visible error state and retry action when tool catalog loading fails", async () => {
    const user = userEvent.setup();
    toolsApiMock.getToolCatalog.mockRejectedValueOnce(new Error("catalog unavailable"));

    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <Manager />
      </QueryClientProvider>,
    );

    expect(await screen.findByText("Failed to load tool catalog.")).toBeInTheDocument();
    expect(screen.getByText("Tool management is unavailable until the catalog can be loaded.")).toBeInTheDocument();
    expect(toolsApiMock.detectTools).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "Retry" }));

    await waitFor(() => {
      expect(toolsApiMock.getToolCatalog).toHaveBeenCalledTimes(2);
    });
  });

  it("hides tools that are not installable on the current platform from the available list", async () => {
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
        <Manager />
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(toolsApiMock.detectTools).toHaveBeenCalledWith(
        ["codex-cli"],
        "D:\\AgenticTools",
        false,
      );
    });

    await userEvent.click(screen.getAllByRole("tab")[1]);
    expect(await screen.findByText("Codex (CLI)")).toBeInTheDocument();
    expect(screen.queryByText("Gemini CLI")).not.toBeInTheDocument();
  });
});
