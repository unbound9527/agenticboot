// Hook for listening to install progress events

import { useState, useEffect, useCallback } from 'react';
import { toolsApi } from '@/lib/api/tools';
import type { InstallProgress } from '@/types/tools';

export function useInstallProgress() {
  const [progressMap, setProgressMap] = useState<
    Map<string, InstallProgress>
  >(new Map());

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    toolsApi.onInstallProgress((progress) => {
      setProgressMap((prev) => {
        const next = new Map(prev);
        next.set(progress.toolId, progress);
        return next;
      });
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  // Clear progress when starting a new install
  const resetProgress = useCallback(() => {
    setProgressMap(new Map());
  }, []);

  const getToolProgress = useCallback(
    (toolId: string): InstallProgress | null => {
      return progressMap.get(toolId) ?? null;
    },
    [progressMap]
  );

  const allComplete = [...progressMap.values()].every(
    (p) => p.phase === 'complete' || p.phase === 'skipped' || p.phase === 'error'
  );

  const hasErrors = [...progressMap.values()].some(
    (p) => p.phase === 'error'
  );

  const completedCount = [...progressMap.values()].filter(
    (p) => p.phase === 'complete' || p.phase === 'skipped'
  ).length;

  const totalCount = progressMap.size;

  return {
    progressMap,
    getToolProgress,
    allComplete,
    hasErrors,
    completedCount,
    totalCount,
    resetProgress,
  };
}
