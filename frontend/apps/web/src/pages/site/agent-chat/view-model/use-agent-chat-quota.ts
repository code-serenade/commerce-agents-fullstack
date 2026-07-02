import { useQuery } from "@tanstack/react-query"
import {
  currentTeamUsageQuotaQueryKey,
  getCurrentTeamUsageQuotaApi,
} from "@/api"

export function useAgentChatQuota({ enabled }: { enabled: boolean }) {
  const quotaQuery = useQuery({
    queryKey: currentTeamUsageQuotaQueryKey,
    queryFn: () =>
      getCurrentTeamUsageQuotaApi({
        suppressGlobalError: true,
      }),
    enabled,
    staleTime: 30_000,
  })

  return {
    quota: quotaQuery.data ?? null,
    isLoadingQuota: quotaQuery.isLoading,
    refreshQuotaAfter: async (action: () => Promise<void>) => {
      await action()
      if (enabled) {
        void quotaQuery.refetch()
      }
    },
  }
}
