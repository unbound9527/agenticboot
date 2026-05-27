import type { AppId } from "@/lib/api/types";

export function isClaudeFamilyApp(appId: AppId): boolean {
  return appId === "claude" || appId === "claude-desktop";
}
