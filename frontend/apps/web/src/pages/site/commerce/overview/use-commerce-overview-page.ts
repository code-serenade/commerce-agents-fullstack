import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import { useQuery } from "@tanstack/react-query"
import {
  getAlertSummaryApi,
  getPlatformSummaryApi,
  listCampaignsApi,
  listExecutionQueueApi,
  listProductsApi,
  listToolApprovalsApi,
  listWorkspacesApi,
  type AlertSummaryResponse,
  type CampaignResponse,
  type ExecutionQueueResponse,
  type PlatformSummaryItem,
  type ProductResponse,
  type ToolApprovalResponse,
  type WorkspaceResponse,
} from "@/api"
import { buildOverviewMetricCards } from "./overview-metrics"

export function useCommerceOverviewPage() {
  const { t } = useTranslation()
  const overviewQuery = useQuery({
    queryKey: ["commerce", "overview"],
    queryFn: async () => {
      const [
        products,
        campaigns,
        workspaces,
        platforms,
        executionQueue,
        approvals,
        alerts,
      ] = await Promise.all([
        listProductsApi({ status: "active" }),
        listCampaignsApi({ pageSize: 20 }),
        listWorkspacesApi({ pageSize: 20 }),
        getPlatformSummaryApi(),
        listExecutionQueueApi({ pageSize: 20, status: "pending" }),
        listToolApprovalsApi({ pageSize: 20, status: "pending" }),
        getAlertSummaryApi(),
      ])

      return {
        alerts,
        approvals: approvals.items,
        campaigns: campaigns.campaigns,
        executionQueue: executionQueue.items,
        platforms: platforms.platforms,
        products,
        workspaces: workspaces.workspaces,
      }
    },
  })
  const data = overviewQuery.data
  const metricCards = useMemo(
    () => buildOverviewMetricCards(t, data),
    [data, t]
  )

  return {
    alerts: data?.alerts,
    approvals: data?.approvals ?? [],
    campaigns: data?.campaigns ?? [],
    executionQueue: data?.executionQueue ?? [],
    loading: overviewQuery.isLoading,
    metricCards,
    platforms: data?.platforms ?? [],
    products: data?.products ?? [],
    workspaces: data?.workspaces ?? [],
  }
}

export interface CommerceOverviewData {
  alerts: AlertSummaryResponse
  approvals: ToolApprovalResponse[]
  campaigns: CampaignResponse[]
  executionQueue: ExecutionQueueResponse[]
  platforms: PlatformSummaryItem[]
  products: ProductResponse[]
  workspaces: WorkspaceResponse[]
}

export type CommerceOverviewProduct = ProductResponse
export type CommerceOverviewCampaign = CampaignResponse
export type CommerceOverviewWorkspace = WorkspaceResponse
export type CommerceOverviewPlatform = PlatformSummaryItem
export type CommerceOverviewExecution = ExecutionQueueResponse
export type CommerceOverviewApproval = ToolApprovalResponse
export type CommerceOverviewAlertSummary = AlertSummaryResponse
