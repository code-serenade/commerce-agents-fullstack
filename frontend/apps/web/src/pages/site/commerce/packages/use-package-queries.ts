import { useMemo } from "react"
import { useQuery } from "@tanstack/react-query"
import {
  getPackageApi,
  listWorkspacesApi,
  listPackageCatalogApi,
  listPackageHistoryApi,
  listPackageLocksApi,
  listWorkspaceRunsApi,
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
  const workspacesQuery = useQuery({
    queryKey: [...packagesQueryKey, "workspaces"],
    queryFn: () =>
      listWorkspacesApi({
        pageSize: 50,
        status: "active",
      }),
  })
  const workspaces = useMemo(
    () => workspacesQuery.data?.workspaces ?? [],
    [workspacesQuery.data?.workspaces]
  )
  const effectiveWorkspaceId = useMemo(
    () =>
      workspaces.find((workspace) => workspace.id === workspaceId)?.id ??
      workspaces[0]?.id ??
      "",
    [workspaceId, workspaces]
  )
  const selectedWorkspace = useMemo(
    () =>
      workspaces.find((workspace) => workspace.id === effectiveWorkspaceId) ?? null,
    [effectiveWorkspaceId, workspaces]
  )
  const runsQuery = useQuery({
    queryKey: [...packagesQueryKey, effectiveWorkspaceId, "runs"],
    enabled: Boolean(effectiveWorkspaceId),
    queryFn: () =>
      listWorkspaceRunsApi(effectiveWorkspaceId, {
        pageSize: 50,
      }),
  })
  const runs = useMemo(
    () => runsQuery.data?.runs ?? [],
    [runsQuery.data?.runs]
  )
  const effectiveRunId = useMemo(
    () => runs.find((run) => run.id === runId)?.id ?? runs[0]?.id ?? "",
    [runId, runs]
  )
  const selectedRun = useMemo(
    () => runs.find((run) => run.id === effectiveRunId) ?? null,
    [effectiveRunId, runs]
  )
  const locksQuery = useQuery({
    queryKey: [...packagesQueryKey, effectiveWorkspaceId, effectiveRunId, "locks"],
    enabled: Boolean(effectiveWorkspaceId && effectiveRunId),
    queryFn: () => listPackageLocksApi(effectiveWorkspaceId, effectiveRunId),
  })

  return {
    effectiveRunId,
    effectiveWorkspaceId,
    history: historyQuery.data?.history ?? [],
    loading:
      catalogQuery.isLoading ||
      packageDetailQuery.isLoading ||
      historyQuery.isLoading ||
      locksQuery.isLoading ||
      workspacesQuery.isLoading ||
      runsQuery.isLoading,
    locks: locksQuery.data?.locks ?? [],
    packages,
    selectedPackage: selectedPackageDetail,
    selectedRun,
    selectedWorkspace,
    runs,
    workspaces,
  }
}
