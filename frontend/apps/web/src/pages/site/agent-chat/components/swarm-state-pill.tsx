import { useTranslation } from "react-i18next"
import {
  getStatePillLabel,
  type SwarmState,
} from "../view-model/swarm-panel-view-model"

export function StatePill({ state }: { state: SwarmState }) {
  const { t } = useTranslation()

  return (
    <span
      className={`shrink-0 rounded-full px-2 py-0.5 text-[11px] font-medium ${
        state === "done"
          ? "bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-200"
          : state === "working"
            ? "bg-blue-50 text-blue-700 dark:bg-blue-950/40 dark:text-blue-200"
            : state === "queued" || state === "reworking"
              ? "bg-amber-50 text-amber-700 dark:bg-amber-950/40 dark:text-amber-200"
              : state === "failed"
                ? "bg-destructive/10 text-destructive"
                : "bg-secondary text-secondary-foreground"
      }`}
    >
      {getStatePillLabel(state, t)}
    </span>
  )
}
