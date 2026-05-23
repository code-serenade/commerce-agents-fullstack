import {
  Activity,
  PanelRightClose,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"
import { getDispatchWorkbenchModel } from "../view-model/agent-dispatch-workbench-view-model"
import type { AgentRunFeedbackState } from "../view-model/use-agent-run-feedback"
import { AgentRunFeedbackPanel } from "./agent-run-feedback-panel"
import { DispatchStepDock } from "./agent-dispatch-workbench-parts"
import { DispatchWorkbenchContent } from "./agent-dispatch-workbench-content"

export function AgentDispatchProcessPanel({
  feedback,
  isStreaming,
  onCollapse,
  runtimeEvents,
  task,
}: {
  feedback: AgentRunFeedbackState
  isStreaming: boolean
  onCollapse: () => void
  runtimeEvents: AgentRuntimeEvent[]
  task?: string
}) {
  const { t } = useTranslation()
  const model = getDispatchWorkbenchModel(
    { isStreaming, runtimeEvents, task },
    t
  )

  return (
    <aside className="h-full min-h-0 min-w-0 overflow-hidden bg-background">
      <div className="flex h-full min-h-0 flex-col overflow-hidden bg-background">
        <header className="sticky top-0 z-10 shrink-0 border-b border-border bg-background/95 px-4 py-3 backdrop-blur">
          <div className="flex items-start justify-between gap-3">
            <div className="flex min-w-0 items-center gap-3">
              <div className="flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-card text-foreground">
                <Activity className="size-4" />
              </div>
              <div className="min-w-0">
                <h2 className="truncate text-sm font-semibold">
                  {t("agentChat.thread.dispatchPanel.workbenchTitle")}
                </h2>
                <div className="mt-1.5 flex min-w-0 flex-wrap items-center gap-2 text-xs text-muted-foreground">
                  <span
                    className={`size-2.5 rounded-full border-2 border-white shadow-sm ${
                      isStreaming ? "animate-pulse bg-emerald-500" : "bg-emerald-500"
                    }`}
                  />
                  <span className="text-foreground">
                    {t("agentChat.workbench.progress")} {model.progress.completed}/{model.progress.total}
                  </span>
                  <span className="hidden sm:inline">|</span>
                  <span className="min-w-0 truncate">{model.progress.stage}</span>
                </div>
              </div>
            </div>
            <div className="rounded-md border border-border bg-card p-1">
              <button
                type="button"
                onClick={onCollapse}
                className="flex size-9 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                title={t("agentChat.workbench.collapse")}
                aria-label={t("agentChat.workbench.collapse")}
              >
                <PanelRightClose className="size-4" />
              </button>
            </div>
          </div>
        </header>

        <div className="min-h-0 flex-1 overflow-x-hidden overflow-y-auto p-3 [scrollbar-gutter:stable]">
          <DispatchWorkbenchContent model={model} />

          <section className="mt-3 grid gap-2 min-[1500px]:grid-cols-2">
            <div className="rounded-lg border border-border bg-card p-3">
              <p className="text-xs font-semibold text-foreground">
                {t("agentChat.thread.dispatchPanel.studentReadTitle")}
              </p>
              <p className="mt-1.5 text-xs/5 text-muted-foreground">
                {model.summary}
              </p>
            </div>
            <AgentRunFeedbackPanel compact feedback={feedback} />
          </section>
        </div>

        <div className="max-h-40 shrink-0 overflow-x-hidden overflow-y-auto border-t border-border [scrollbar-gutter:stable]">
          <DispatchStepDock steps={model.steps} />
        </div>
      </div>
    </aside>
  )
}
