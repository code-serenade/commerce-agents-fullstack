import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { usePackageActions } from "./use-package-actions"
import { buildPackageMetricCards } from "./package-metrics"
import { usePackageQueries } from "./use-package-queries"

export function usePackagesPage() {
  const { t } = useTranslation()
  const [keyword, setKeyword] = useState("")
  const [packageType, setPackageType] = useState("")
  const [selectedPackageKey, setSelectedPackageKey] = useState<string | null>(null)
  const [workspaceId, setWorkspaceId] = useState("")
  const [runId, setRunId] = useState("")
  const [dialog, setDialog] = useState<"package" | "lock" | null>(null)
  const queries = usePackageQueries({
    keyword,
    packageType,
    runId,
    selectedPackageKey,
    workspaceId,
  })
  const actions = usePackageActions({
    runId,
    selectedPackage: queries.selectedPackage,
    setDialog,
    setSelectedPackageKey,
    t,
    workspaceId,
  })
  const metricCards = useMemo(
    () =>
      buildPackageMetricCards(t, {
        locks: queries.locks,
        packages: queries.packages,
      }),
    [queries.locks, queries.packages, t]
  )

  return {
    ...actions,
    ...queries,
    dialog,
    keyword,
    metricCards,
    packageType,
    runId,
    setDialog,
    setKeyword,
    setPackageType,
    setRunId,
    setSelectedPackageKey,
    setWorkspaceId,
    workspaceId,
  }
}

export function packageKey(item: { packageType: string; packageId: string }) {
  return `${item.packageType}:${item.packageId}`
}

export type PackagesPageState = ReturnType<typeof usePackagesPage>
