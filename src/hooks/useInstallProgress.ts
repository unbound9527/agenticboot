// Hook for listening to install progress events

import { useState, useEffect, useCallback } from "react";
import { toolsApi } from "@/lib/api/tools";
import type { InstallProgress } from "@/types/tools";

const TERMINAL_PHASES = new Set<InstallProgress["phase"]>([
  "complete",
  "error",
  "skipped",
]);
const PROGRESS_SMOOTH_INTERVAL_MS = 50;
const PROGRESS_SMOOTH_STEP_DIVISOR = 5;

function clampPercent(percent: number): number {
  return Math.max(0, Math.min(100, Math.round(percent)));
}

function getNextDisplayedPercent(current: number, target: number): number {
  if (target <= current) {
    return target;
  }

  const remaining = target - current;
  const step = Math.max(1, Math.ceil(remaining / PROGRESS_SMOOTH_STEP_DIVISOR));
  return Math.min(target, current + step);
}

export function useInstallProgress() {
  const [targetProgressMap, setTargetProgressMap] = useState<
    Map<string, InstallProgress>
  >(new Map());
  const [displayedProgressMap, setDisplayedProgressMap] = useState<
    Map<string, InstallProgress>
  >(new Map());

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;

    toolsApi.onInstallProgress((progress) => {
      const normalizedProgress = {
        ...progress,
        percent: clampPercent(progress.percent),
      };

      setTargetProgressMap((prev) => {
        const next = new Map(prev);
        next.set(normalizedProgress.toolId, normalizedProgress);
        return next;
      });

      setDisplayedProgressMap((prev) => {
        const next = new Map(prev);
        const current = next.get(normalizedProgress.toolId);
        const shouldSnapImmediately = TERMINAL_PHASES.has(normalizedProgress.phase);
        const displayedPercent = shouldSnapImmediately
          ? normalizedProgress.percent
          : current
            ? Math.min(current.percent, normalizedProgress.percent)
            : 0;

        next.set(normalizedProgress.toolId, {
          ...normalizedProgress,
          percent: displayedPercent,
        });
        return next;
      });
    })
      .then((fn) => {
        if (disposed) {
          fn();
          return;
        }

        unlisten = fn;
      })
      .catch(() => {
        // Ignore subscription errors so the install UI can fail gracefully.
      });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    const hasPendingSmoothing = [...targetProgressMap.entries()].some(
      ([toolId, targetProgress]) => {
        if (TERMINAL_PHASES.has(targetProgress.phase)) {
          return false;
        }

        const displayedProgress = displayedProgressMap.get(toolId);
        return (displayedProgress?.percent ?? 0) < targetProgress.percent;
      },
    );

    if (!hasPendingSmoothing) {
      return;
    }

    const timer = window.setInterval(() => {
      setDisplayedProgressMap((prev) => {
        let changed = false;
        const next = new Map(prev);

        targetProgressMap.forEach((targetProgress, toolId) => {
          if (TERMINAL_PHASES.has(targetProgress.phase)) {
            return;
          }

          const currentProgress = next.get(toolId);
          if (!currentProgress) {
            next.set(toolId, { ...targetProgress, percent: 0 });
            changed = true;
            return;
          }

          const nextPercent = getNextDisplayedPercent(
            currentProgress.percent,
            targetProgress.percent,
          );

          if (nextPercent !== currentProgress.percent) {
            next.set(toolId, {
              ...targetProgress,
              percent: nextPercent,
            });
            changed = true;
          }
        });

        return changed ? next : prev;
      });
    }, PROGRESS_SMOOTH_INTERVAL_MS);

    return () => {
      window.clearInterval(timer);
    };
  }, [displayedProgressMap, targetProgressMap]);

  // Clear progress when starting a new install
  const resetProgress = useCallback(() => {
    setTargetProgressMap(new Map());
    setDisplayedProgressMap(new Map());
  }, []);

  const getToolProgress = useCallback(
    (toolId: string): InstallProgress | null => {
      return displayedProgressMap.get(toolId) ?? null;
    },
    [displayedProgressMap]
  );

  const hasAnyProgress = displayedProgressMap.size > 0;

  const allComplete = hasAnyProgress && [...displayedProgressMap.values()].every(
    (p) => p.phase === "complete" || p.phase === "skipped" || p.phase === "error"
  );

  const hasErrors = [...displayedProgressMap.values()].some(
    (p) => p.phase === "error"
  );

  const completedCount = [...displayedProgressMap.values()].filter(
    (p) => p.phase === "complete" || p.phase === "skipped"
  ).length;

  const totalCount = displayedProgressMap.size;

  return {
    progressMap: displayedProgressMap,
    getToolProgress,
    hasAnyProgress,
    allComplete,
    hasErrors,
    completedCount,
    totalCount,
    resetProgress,
  };
}
