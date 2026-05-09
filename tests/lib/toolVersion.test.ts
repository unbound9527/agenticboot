import { describe, expect, it } from "vitest";
import { formatInstalledVersion } from "@/lib/tools/version";

describe("formatInstalledVersion", () => {
  it("extracts a short semver label from verbose tool output", () => {
    expect(
      formatInstalledVersion(
        "Hermes Agent v0.12.0 (2026.4.30) Project: D:\\projects\\hermes-agent",
      ),
    ).toBe("v0.12.0");
  });

  it("avoids duplicating the v prefix when the version already includes it", () => {
    expect(formatInstalledVersion("v22.15.0")).toBe("v22.15.0");
  });

  it("returns the original text when no semver-like version can be found", () => {
    expect(formatInstalledVersion("nightly build")).toBe("nightly build");
  });
});
