import { useQuery } from "@tanstack/react-query"
import {
  getExternalSearchStatusApi,
  getPlatformSummaryApi,
  getPlatformSyncStatusApi,
  listChannelConnectionsApi,
  listPlatformExecutionAdaptersApi,
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
  const executionAdaptersQuery = useQuery({
    queryKey: [...platformQueryKey, "execution-adapters"],
    queryFn: listPlatformExecutionAdaptersApi,
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
  const externalSearchStatusQuery = useQuery({
    queryKey: [...platformQueryKey, "external-search-status"],
    queryFn: getExternalSearchStatusApi,
  })

  return {
    channelConnections: channelConnectionsQuery.data?.connections ?? [],
    externalSearchStatus: externalSearchStatusQuery.data ?? null,
    loading:
      supportedPlatformsQuery.isLoading ||
      platformConnectionsQuery.isLoading ||
      executionAdaptersQuery.isLoading ||
      supportedChannelsQuery.isLoading ||
      channelConnectionsQuery.isLoading ||
      metricsQuery.isLoading ||
      syncStatusQuery.isLoading ||
      summaryQuery.isLoading ||
      externalSearchStatusQuery.isLoading,
    metrics: metricsQuery.data?.metrics ?? [],
    platformExecutionAdapters: executionAdaptersQuery.data?.adapters ?? [],
    platformConnections: platformConnectionsQuery.data?.connections ?? [],
    summaries: summaryQuery.data?.platforms ?? [],
    supportedChannels: supportedChannelsQuery.data?.channels ?? [],
    supportedPlatforms: supportedPlatformsQuery.data?.platforms ?? [],
    syncStatuses: syncStatusQuery.data?.platforms ?? [],
  }
}
