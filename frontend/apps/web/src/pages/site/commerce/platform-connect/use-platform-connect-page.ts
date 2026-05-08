import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { usePlatformActions } from "./use-platform-actions"
import { buildPlatformMetricCards } from "./platform-metrics"
import { usePlatformQueries } from "./use-platform-queries"

export function usePlatformConnectPage() {
  const { t } = useTranslation()
  const [dialog, setDialog] = useState<PlatformDialogName | null>(null)
  const queries = usePlatformQueries()
  const actions = usePlatformActions({ setDialog, t })
  const metricCards = useMemo(
    () =>
      buildPlatformMetricCards(t, {
        channelConnections: queries.channelConnections,
        metrics: queries.metrics,
        platformConnections: queries.platformConnections,
        summaries: queries.summaries,
      }),
    [
      queries.channelConnections,
      queries.metrics,
      queries.platformConnections,
      queries.summaries,
      t,
    ]
  )

  return {
    ...actions,
    ...queries,
    dialog,
    metricCards,
    setDialog,
  }
}

export type PlatformDialogName = "platform" | "channel" | "metric"
export type PlatformConnectPageState = ReturnType<typeof usePlatformConnectPage>
