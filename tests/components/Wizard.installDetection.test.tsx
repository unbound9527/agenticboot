import { QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
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
    toolsApiMock.onInstallProgress.mockResolvedValue(() => {});
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
        "D:\\AITools",
      );
    });

    await waitFor(() => {
      expect(screen.getAllByRole("checkbox")).toHaveLength(8);
    });

    fireEvent.change(screen.getByDisplayValue("D:\\AITools"), {
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

    fireEvent.change(screen.getByDisplayValue("D:\\AITools"), {
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

    fireEvent.change(screen.getByDisplayValue("D:\\AITools"), {
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
