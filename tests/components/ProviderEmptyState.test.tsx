import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ProviderEmptyState } from "@/components/providers/ProviderEmptyState";

describe("ProviderEmptyState", () => {
  it("为 claude-desktop 显示导入当前配置提示和按钮", () => {
    const handleImport = vi.fn();

    render(
      <ProviderEmptyState
        appId="claude-desktop"
        onImport={handleImport}
        onCreate={vi.fn()}
      />,
    );

    expect(
      screen.getByText("provider.noProvidersDescriptionSnippet"),
    ).toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", {
        name: "provider.importCurrent",
      }),
    );

    expect(handleImport).toHaveBeenCalledTimes(1);
  });
});
