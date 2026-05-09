import { describe, expect, it, vi } from "vitest";
import { withTimeout } from "@/lib/api/tools";

describe("withTimeout", () => {
  it("rejects when the wrapped promise does not settle before the timeout", async () => {
    vi.useFakeTimers();

    const pending = new Promise<never>(() => {});
    const wrapped = withTimeout(pending, 50, "timed out");
    const expectation = expect(wrapped).rejects.toThrow("timed out");

    await vi.advanceTimersByTimeAsync(51);

    await expectation;
    vi.useRealTimers();
  });

  it("resolves when the wrapped promise settles before the timeout", async () => {
    vi.useFakeTimers();

    const wrapped = withTimeout(Promise.resolve("ok"), 50, "timed out");
    await vi.runAllTimersAsync();

    await expect(wrapped).resolves.toBe("ok");
    vi.useRealTimers();
  });
});
