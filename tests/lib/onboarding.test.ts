import { describe, expect, it } from "vitest";
import { shouldShowStartupWizard } from "@/lib/tools/onboarding";

describe("shouldShowStartupWizard", () => {
  it("shows the wizard only when no tools are installed and the user has not seen it yet", () => {
    expect(shouldShowStartupWizard(false, false)).toBe(true);
    expect(shouldShowStartupWizard(true, false)).toBe(false);
    expect(shouldShowStartupWizard(false, true)).toBe(false);
    expect(shouldShowStartupWizard(true, true)).toBe(false);
  });
});
