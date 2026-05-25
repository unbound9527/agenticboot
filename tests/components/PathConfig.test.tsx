import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { PathConfig } from "@/components/tools/PathConfig";

describe("PathConfig", () => {
  it("shows D:\\AgenticTools as the default root, placeholder, and preview", () => {
    const { container } = render(
      <PathConfig onNext={vi.fn()} onBack={vi.fn()} />,
    );

    expect(screen.getByDisplayValue("D:\\AgenticTools")).toHaveAttribute(
      "placeholder",
      "D:\\AgenticTools",
    );
    expect(container.querySelector("pre")).toHaveTextContent("D:\\AgenticTools");
  });
});
