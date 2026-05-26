// Hook for listening to resolve-progress events during plan resolution

import { useState, useEffect } from "react";
import { toolsApi } from "@/lib/api/tools";
import type { ResolveProgress } from "@/types/tools";

export function useResolveProgress() {
  const [progressMap, setProgressMap] = useState<Map<string, ResolveProgress>>(new Map());
  const [isResolving, setIsResolving] = useState(false);
  const [completed, setCompleted] = useState(false);

  useEffect(() => {
    let unlistenProgress: (() => void) | undefined;
    let unlistenComplete: (() => void) | undefined;
    let disposed = false;

    toolsApi
      .onResolveProgress((progress) => {
        if (disposed) return;
        setIsResolving(true);
        setProgressMap((prev) => {
          const next = new Map(prev);
          next.set(progress.toolId, progress);
          return next;
        });
      })
      .then((fn) => {
        if (disposed) {
          fn();
          return;
        }
        unlistenProgress = fn;
      })
      .catch(() => {});

    toolsApi
      .onResolveComplete(() => {
        if (disposed) return;
        setIsResolving(false);
        setCompleted(true);
      })
      .then((fn) => {
        if (disposed) {
          fn();
          return;
        }
        unlistenComplete = fn;
      })
      .catch(() => {});

    return () => {
      disposed = true;
      unlistenProgress?.();
      unlistenComplete?.();
    };
  }, []);

  const getProgress = (toolId: string): ResolveProgress | null => {
    return progressMap.get(toolId) ?? null;
  };

  const reset = () => {
    setProgressMap(new Map());
    setIsResolving(false);
    setCompleted(false);
  };

  return {
    progressMap,
    isResolving,
    completed,
    getProgress,
    reset,
  };
}