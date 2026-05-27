import {
  CheckCircle2,
  CircleAlert,
  FileText,
  Loader2,
  MonitorCog,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import type {
  DispatchWorkbenchStep,
} from "../view-model/agent-dispatch-workbench-view-model"
import type {
  AgentDispatchFact,
} from "../view-model/agent-process-kind-view-model"
import type { SwarmState } from "../view-model/swarm-panel-view-model"
import { formatAgentNumber, getDockStateLabel } from "../view-model/swarm-panel-view-model"

export function DispatchPreviewDocument({
  filename,
  lines,
}: {
  filename: string
  lines: string[]
}) {
  const { t } = useTranslation()

  return (
    <div className="min-h-0 bg-background p-2.5 min-[1500px]:p-3">
      <div className="flex h-full min-h-36 flex-col overflow-hidden rounded-md border border-border bg-card">
        <div className="flex items-center justify-between gap-3 border-b border-border bg-background px-4 py-2.5">
          <div className="flex min-w-0 items-center gap-2">
            <FileText className="size-4 shrink-0 text-muted-foreground" />
            <span className="truncate text-sm font-medium">{filename}</span>
          </div>
          <span className="shrink-0 rounded-full bg-secondary px-2 py-1 text-[11px] text-secondary-foreground">
            {t("agentChat.workbench.preview")}
          </span>
        </div>
        <div className="min-h-0 flex-1 overflow-x-hidden overflow-y-auto py-2 font-mono text-[11px]/5 [scrollbar-gutter:stable]">
          {lines.map((line, index) => (
            <div
              key={`${index}-${line}`}
              className="grid grid-cols-[2.5rem_minmax(0,1fr)] gap-3 px-3 py-0.5"
            >
              <span className="text-right text-muted-foreground/70 tabular-nums select-none">
                {String(index + 1).padStart(2, "0")}
              </span>
              <span className="min-w-0 wrap-break-word text-foreground/85">
                {line}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

export function DispatchFactChips({ facts }: { facts: AgentDispatchFact[] }) {
  return (
    <div className="grid min-w-0 gap-1.5 min-[1500px]:grid-cols-2">
      {facts.map((fact, index) => {
        const Icon = fact.tone === "warning" || fact.tone === "error"
          ? CircleAlert
          : CheckCircle2

        return (
          <article
            key={`${index}-${fact.label}`}
            className="flex min-w-0 items-start gap-2 rounded-md border border-border bg-card px-2.5 py-1.5 text-xs"
            title={`${fact.label} ${fact.detail}`}
          >
            <Icon className="mt-0.5 size-3.5 shrink-0 text-muted-foreground" />
            <div className="min-w-0 flex-1">
              <p className="font-medium text-foreground">{fact.label}</p>
              <p className="mt-0.5 wrap-break-word text-[11px] text-muted-foreground">
                {fact.detail}
              </p>
            </div>
          </article>
        )
      })}
    </div>
  )
}

export function DispatchStepDock({ steps }: { steps: DispatchWorkbenchStep[] }) {
  const { t } = useTranslation()

  return (
    <div className="shrink-0 bg-background p-2">
      <div className="grid min-w-0 grid-cols-2 gap-1.5 min-[1500px]:grid-cols-3">
        {steps.map((step) => {
          const isActive = step.state === "working" || step.state === "failed"

          return (
            <article
              key={step.id}
              title={`${formatAgentNumber(step.index)} ${step.title} ${getDockStateLabel(step.state, t)} ${step.detail}`}
              className={`flex min-w-0 items-start gap-2 rounded-md border px-2 py-1.5 transition-colors ${
                isActive
                  ? "border-foreground bg-background shadow-sm"
                  : "border-border bg-background/70 text-muted-foreground"
              }`}
            >
              <div
                className={`flex size-7 shrink-0 items-center justify-center rounded-md border border-border bg-background ${
                  isActive ? "text-foreground" : "text-muted-foreground"
                }`}
              >
                <StepStateIcon state={step.state} compact />
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex min-w-0 items-center gap-1.5">
                  <span className="shrink-0 font-mono text-xs font-semibold text-muted-foreground">
                    {formatAgentNumber(step.index)}
                  </span>
                  <p className="truncate text-xs font-semibold">{step.title}</p>
                </div>
                <div className="mt-0.5 flex items-center gap-1.5 text-[11px] text-muted-foreground">
                  <span className="truncate">
                    {getDockStateLabel(step.state, t)}
                  </span>
                </div>
                <p className="mt-1 wrap-break-word text-[11px]/4 text-muted-foreground">
                  {step.detail}
                </p>
              </div>
            </article>
          )
        })}
      </div>
    </div>
  )
}

export function StepStateIcon({
  compact = false,
  state,
}: {
  compact?: boolean
  state: SwarmState
}) {
  const className = compact ? "size-3.5" : "size-4"
  if (state === "done") return <CheckCircle2 className={`${className} text-emerald-600`} />
  if (state === "failed") return <CircleAlert className={`${className} text-destructive`} />
  if (state === "working" || state === "reworking") {
    return <Loader2 className={`${className} animate-spin text-emerald-600`} />
  }
  if (state === "boundary") return <MonitorCog className={`${className} text-amber-600`} />
  return <MonitorCog className={`${className} text-muted-foreground`} />
}
