import { MonitorCog } from "lucide-react"
import { useTranslation } from "react-i18next"
import type {
  DispatchWorkbenchModel,
} from "../view-model/agent-dispatch-workbench-view-model"
import { formatAgentNumber, getDockStateLabel } from "../view-model/swarm-panel-view-model"
import {
  DispatchFactChips,
  DispatchPreviewDocument,
  StepStateIcon,
} from "./agent-dispatch-workbench-parts"
import { StatePill } from "./swarm-state-pill"

export function DispatchWorkbenchContent({
  model,
}: {
  model: DispatchWorkbenchModel
}) {
  const { t } = useTranslation()

  return (
    <section className="grid min-h-0 overflow-hidden rounded-lg border border-border bg-background">
      <div className="min-w-0 bg-background">
        <div className="flex items-center justify-between gap-3 border-b border-border bg-background px-4 py-3 min-[1500px]:px-5">
          <div className="flex min-w-0 items-center gap-2">
            <div className="flex size-8 shrink-0 items-center justify-center rounded-md border border-border bg-card">
              <MonitorCog className="size-4" />
            </div>
            <div className="min-w-0">
              <p className="truncate font-mono text-sm font-semibold">
                {formatAgentNumber(model.activeStepIndex)} | {model.activeStep.title}
              </p>
              <p className="truncate text-xs text-muted-foreground">
                {model.modeLabel} | {model.filename}
              </p>
            </div>
          </div>
          <StatePill state={model.activeStep.state} />
        </div>

        <div className="grid min-h-0">
          <div className="border-b border-border bg-muted/30 px-3 py-2.5 min-[1500px]:px-4">
            <span className="text-xs font-semibold text-foreground">
              {t("agentChat.workbench.businessInput")}
            </span>
            <p className="mt-1 wrap-break-word text-sm/6 text-foreground">
              {model.request}
            </p>
          </div>

          <DispatchPreviewDocument
            filename={model.filename}
            lines={model.previewLines}
          />

          <div className="grid gap-2 border-t border-border bg-background p-2 min-[1500px]:grid-cols-[minmax(0,1fr)_8rem]">
            <div className="min-w-0 rounded-md border border-border bg-card px-3 py-2">
              <div className="flex min-w-0 items-center gap-2 text-xs text-muted-foreground">
                <StepStateIcon state={model.activeStep.state} />
                <span className="truncate">
                  {model.activeStep.title} | {getDockStateLabel(model.activeStep.state, t)}
                </span>
              </div>
              <p className="mt-1 wrap-break-word text-[11px] text-muted-foreground">
                {model.activeStep.detail}
              </p>
            </div>
            <div className="rounded-md border border-border bg-card px-3 py-2">
              <p className="text-xs text-muted-foreground">
                {t("agentChat.thread.dispatchPanel.routeType")}
              </p>
              <p className="mt-1 truncate text-sm font-semibold">
                {model.modeLabel}
              </p>
            </div>
            <div className="min-w-0 min-[1500px]:col-span-2">
              <DispatchFactChips facts={model.facts} />
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
