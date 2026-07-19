import { useQuery } from "@tanstack/react-query"
import { listCampaignsApi, listProductsApi } from "@/api"

export function useWorkspaceDialogOptions(enabled: boolean) {
  const productsQuery = useQuery({
    enabled,
    queryKey: ["commerce", "workspaces", "binding-products"],
    queryFn: () => listProductsApi({ pageSize: 100 }),
  })
  const campaignsQuery = useQuery({
    enabled,
    queryKey: ["commerce", "workspaces", "binding-campaigns"],
    queryFn: () => listCampaignsApi({ pageSize: 100 }),
  })

  return {
    campaigns: campaignsQuery.data?.campaigns ?? [],
    products: productsQuery.data ?? [],
  }
}
