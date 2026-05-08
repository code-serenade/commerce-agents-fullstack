import { useMemo } from "react"
import { useQuery } from "@tanstack/react-query"
import {
  getPackageApi,
  listPackageCatalogApi,
  listPackageHistoryApi,
  listPackageLocksApi,
} from "@/api"
import { packageKey } from "./use-packages-page"

export const packagesQueryKey = ["commerce", "packages"]

export function usePackageQueries({
  keyword,
  packageType,
  runId,
  selectedPackageKey,
  workspaceId,
}: {
  keyword: string
  packageType: string
  runId: string
  selectedPackageKey: string | null
  workspaceId: string
}) {
  const catalogQuery = useQuery({
    queryKey: [...packagesQueryKey, keyword, packageType],
    queryFn: () =>
      listPackageCatalogApi({
        includeDisabled: true,
        keyword: keyword || undefined,
        packageType: packageType || undefined,
        pageSize: 100,
      }),
  })
  const packages = useMemo(
    () => catalogQuery.data?.packages ?? [],
    [catalogQuery.data?.packages]
  )
  const selectedPackage = useMemo(() => {
    if (!packages.length) return null

    return (
      packages.find((item) => packageKey(item) === selectedPackageKey) ??
      packages[0]
    )
  }, [packages, selectedPackageKey])
  const packageDetailQuery = useQuery({
    queryKey: [
      ...packagesQueryKey,
      selectedPackage?.packageType,
      selectedPackage?.packageId,
      "detail",
    ],
    enabled: Boolean(selectedPackage),
    queryFn: () =>
      getPackageApi(selectedPackage!.packageType, selectedPackage!.packageId),
  })
  const selectedPackageDetail = packageDetailQuery.data ?? selectedPackage
  const historyQuery = useQuery({
    queryKey: [
      ...packagesQueryKey,
      selectedPackageDetail?.packageType,
      selectedPackageDetail?.packageId,
      "history",
    ],
    enabled: Boolean(selectedPackageDetail),
    queryFn: () =>
      listPackageHistoryApi(
        selectedPackageDetail!.packageType,
        selectedPackageDetail!.packageId
      ),
  })
  const locksQuery = useQuery({
    queryKey: [...packagesQueryKey, workspaceId, runId, "locks"],
    enabled: Boolean(workspaceId && runId),
    queryFn: () => listPackageLocksApi(workspaceId, runId),
  })

  return {
    history: historyQuery.data?.history ?? [],
    loading:
      catalogQuery.isLoading ||
      packageDetailQuery.isLoading ||
      historyQuery.isLoading ||
      locksQuery.isLoading,
    locks: locksQuery.data?.locks ?? [],
    packages,
    selectedPackage: selectedPackageDetail,
  }
}
