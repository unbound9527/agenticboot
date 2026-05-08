// React Query hooks for tool management

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toolsApi } from '@/lib/api/tools';

const TOOLS_KEY = ['installed-tools'] as const;
const NETWORK_KEY = ['network-status'] as const;
const UPDATES_KEY = ['tool-updates'] as const;

export function useInstalledTools() {
  return useQuery({
    queryKey: TOOLS_KEY,
    queryFn: () => toolsApi.getInstalledTools(),
    staleTime: 30_000,
  });
}

export function useHasInstalledTools() {
  return useQuery({
    queryKey: ['has-installed-tools'],
    queryFn: () => toolsApi.hasAnyInstalledTools(),
  });
}

export function useCheckNetwork() {
  return useQuery({
    queryKey: NETWORK_KEY,
    queryFn: () => toolsApi.checkNetwork(),
    retry: false,
    refetchOnWindowFocus: true,
  });
}

export function useInstallRoot() {
  return useQuery({
    queryKey: ['install-root'],
    queryFn: () => toolsApi.getInstallRoot(),
  });
}

export function useToolUpdates() {
  return useQuery({
    queryKey: UPDATES_KEY,
    queryFn: () => toolsApi.checkToolUpdates(),
  });
}

export function useResolveInstallPlan() {
  return useMutation({
    mutationFn: (toolIds: string[]) => toolsApi.resolveInstallPlan(toolIds),
  });
}

export function useExecuteInstallPlan() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      plan,
      rootPath,
    }: {
      plan: Parameters<typeof toolsApi.executeInstallPlan>[0];
      rootPath: string;
    }) => toolsApi.executeInstallPlan(plan, rootPath),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: TOOLS_KEY });
      queryClient.invalidateQueries({ queryKey: ['has-installed-tools'] });
    },
  });
}

export function useUninstallTool() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ toolId, rootPath }: { toolId: string; rootPath: string }) =>
      toolsApi.uninstallTool(toolId, rootPath),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: TOOLS_KEY });
      queryClient.invalidateQueries({ queryKey: ['has-installed-tools'] });
    },
  });
}
