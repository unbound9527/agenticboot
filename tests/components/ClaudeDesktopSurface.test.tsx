import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AppSwitcher } from "@/components/AppSwitcher";
import { APP_ICON_MAP, APP_IDS } from "@/config/appConfig";
import { settingsSchema } from "@/lib/schemas/settings";

vi.mock("@/components/ProviderIcon", () => ({
  ProviderIcon: ({ name }: { name: string }) => <span>{name}</span>,
}));

describe("Claude Desktop surface", () => {
  it("includes claude-desktop in the shared app registry", () => {
    expect(APP_IDS).toContain("claude-desktop");
    expect(APP_ICON_MAP["claude-desktop"].label).toBe("Claude Desktop");
  });

  it("renders Claude Desktop in the app switcher when visible", () => {
    render(
      <AppSwitcher
        activeApp="claude"
        onSwitch={() => {}}
        visibleApps={
          {
            claude: true,
            "claude-desktop": true,
            codex: true,
            gemini: true,
            opencode: true,
            openclaw: true,
            hermes: true,
          } as any
        }
      />,
    );

    expect(
      screen.getByRole("button", { name: /Claude Desktop/i }),
    ).toBeInTheDocument();
  });

  it("accepts claude-desktop visibility and current provider settings", () => {
    const parsed = settingsSchema.parse({
      showInTray: true,
      minimizeToTrayOnClose: false,
      visibleApps: {
        claude: true,
        "claude-desktop": true,
        codex: true,
        gemini: true,
        opencode: true,
        openclaw: true,
      },
      currentProviderClaudeDesktop: "desktop-provider",
    });

    expect(parsed.visibleApps?.["claude-desktop"]).toBe(true);
    expect(parsed.currentProviderClaudeDesktop).toBe("desktop-provider");
  });
});
