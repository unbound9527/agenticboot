import { invoke } from "@tauri-apps/api/core";
import type {
  HermesMemoryKind,
  HermesMemoryLimits,
  HermesModelConfig,
} from "@/types";

/**
 * Hermes Desktop configuration API used by AgenticBoot.
 *
 * AgenticBoot keeps its Hermes surface intentionally minimal: deep
 * configuration (model, agent behavior, env vars, skills, cron, logs,
 * analytics) lives in Hermes Desktop. AgenticBoot reads the `model` section to
 * highlight the active provider and launches Hermes Desktop for everything
 * else. Writes to `model` happen implicitly via `apply_switch_defaults` when
 * the user switches providers.
 */
export const hermesApi = {
  async getModelConfig(): Promise<HermesModelConfig | null> {
    return await invoke("get_hermes_model_config");
  },

  /**
   * Find and launch the Hermes Desktop application.
   * Optional `path` is kept for API compatibility but is unused by the
   * desktop app (no URL deep-linking).
   */
  async openDesktop(path?: string): Promise<void> {
    await invoke("open_hermes_desktop", { path: path ?? null });
  },

  /**
   * Read one of Hermes' memory blobs (`MEMORY.md` or `USER.md`). Returns an
   * empty string when the file hasn't been created yet.
   */
  async getMemory(kind: HermesMemoryKind): Promise<string> {
    return await invoke("get_hermes_memory", { kind });
  },

  /** Atomically overwrite a Hermes memory file. */
  async setMemory(kind: HermesMemoryKind, content: string): Promise<void> {
    await invoke("set_hermes_memory", { kind, content });
  },

  /**
   * Character budgets + enable flags for both memory blobs, read from
   * config.yaml with Hermes defaults as fallback.
   */
  async getMemoryLimits(): Promise<HermesMemoryLimits> {
    return await invoke("get_hermes_memory_limits");
  },

  /**
   * Toggle the on/off flag for one memory blob. Other fields in the `memory:`
   * section (budgets, external provider config) are preserved.
   */
  async setMemoryEnabled(
    kind: HermesMemoryKind,
    enabled: boolean,
  ): Promise<void> {
    await invoke("set_hermes_memory_enabled", { kind, enabled });
  },
};
