import { useQuery } from "@tanstack/react-query"
import {
  getPlatformSummaryApi,
  getPlatformSyncStatusApi,
  listChannelConnectionsApi,
  listPlatformConnectionsApi,
  listStoreMetricsApi,
  listSupportedChannelsApi,
  listSupportedPlatformsApi,
} from "@/api"

export const platformQueryKey = ["commerce", "platform-connect"]

export function usePlatformQueries() {
  const supportedPlatformsQuery = useQuery({
    queryKey: [...platformQueryKey, "supported-platforms"],
    queryFn: listSupportedPlatformsApi,
  })
  const platformConnectionsQuery = useQuery({
    queryKey: [...platformQueryKey, "platform-connections"],
    queryFn: () => listPlatformConnectionsApi(),
  })
  const supportedChannelsQuery = useQuery({
    queryKey: [...platformQueryKey, "supported-channels"],
    queryFn: listSupportedChannelsApi,
  })
  const channelConnectionsQuery = useQuery({
    queryKey: [...platformQueryKey, "channel-connections"],
    queryFn: () => listChannelConnectionsApi(),
  })
  const metricsQuery = useQuery({
    queryKey: [...platformQueryKey, "metrics"],
    queryFn: () => listStoreMetricsApi({ pageSize: 100 }),
  })
  const syncStatusQuery = useQuery({
    queryKey: [...platformQueryKey, "sync-status"],
    queryFn: getPlatformSyncStatusApi,
  })
  const summaryQuery = useQuery({
    queryKey: [...platformQueryKey, "summary"],
    queryFn: getPlatformSummaryApi,
  })

  return {
    channelConnections: channelConnectionsQuery.data?.connections ?? [],
    loading:
      supportedPlatformsQuery.isLoading ||
      platformConnectionsQuery.isLoading ||
      supportedChannelsQuery.isLoading ||
      channelConnectionsQuery.isLoading ||
      metricsQuery.isLoading ||
      syncStatusQuery.isLoading ||
      summaryQuery.isLoading,
    metrics: metricsQuery.data?.metrics ?? [],
    platformConnections: platformConnectionsQuery.data?.connections ?? [],
    summaries: summaryQuery.data?.platforms ?? [],
    supportedChannels: supportedChannelsQuery.data?.channels ?? [],
    supportedPlatforms: supportedPlatformsQuery.data?.platforms ?? [],
    syncStatuses: syncStatusQuery.data?.platforms ?? [],
  }
}
