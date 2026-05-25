import fs from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const iconSvgPath = path.resolve(
  __dirname,
  "..",
  "..",
  "assets",
  "icons",
  "app-icon-design.svg",
);

describe("app icon design source", () => {
  it("defines an explicit transparent canvas for stable icon export", () => {
    const svg = fs.readFileSync(iconSvgPath, "utf8");

    expect(svg).toContain(
      '<rect width="512" height="512" fill="transparent"/>',
    );
  });

  it("scales the main motif up for better small-size legibility", () => {
    const svg = fs.readFileSync(iconSvgPath, "utf8");

    expect(svg).toContain('transform="translate(256,256) scale(1.18)"');
  });

  it("uses a simplified light center instead of the old translucent glow ring", () => {
    const svg = fs.readFileSync(iconSvgPath, "utf8");

    expect(svg).not.toContain('r="28" fill="#FFF5D8" fill-opacity="0.72"');
  });
});
