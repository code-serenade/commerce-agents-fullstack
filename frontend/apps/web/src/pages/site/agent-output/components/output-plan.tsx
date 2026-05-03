import { CheckCircle2 } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentOutputAction, AgentOutputStep } from "../types"

export interface OutputPlanProps {
  actions: AgentOutputAction[]
  steps: AgentOutputStep[]
}

export function OutputPlan({ actions, steps }: OutputPlanProps) {
  const { t } = useTranslation()

  return (
    <section className="grid gap-4 xl:grid-cols-[1fr_20rem]">
      <div className="rounded-lg border border-(--app-border) bg-(--app-surface) p-5">
        <div className="mb-5">
          <p className="text-sm font-medium text-(--app-muted-text)">
            {t("agentOutput.plan.eyebrow")}
          </p>
          <h2 className="mt-1 text-xl font-semibold text-(--app-text)">
            {t("agentOutput.plan.title")}
          </h2>
        </div>
        <div className="grid gap-3 md:grid-cols-2">
          {steps.map((step, index) => (
            <article
              key={step.title}
              className="rounded-lg border border-(--app-border) bg-(--app-bg) p-4"
            >
              <div className="flex items-start gap-3">
                <div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-(--app-active-bg) text-sm font-semibold text-(--app-active-text)">
                  {index + 1}
                </div>
                <div className="min-w-0">
                  <h3 className="text-base font-semibold text-(--app-text)">
                    {step.title}
                  </h3>
                  <p className="mt-1 text-xs text-(--app-muted-text)">
                    {step.owner} · {step.timing}
                  </p>
                </div>
              </div>
              <p className="mt-3 text-sm/6 text-(--app-muted-text)">
                {step.detail}
              </p>
            </article>
          ))}
        </div>
      </div>

      <aside className="rounded-lg border border-(--app-border) bg-(--app-surface) p-5">
        <p className="text-sm font-medium text-(--app-muted-text)">
          {t("agentOutput.plan.actionsTitle")}
        </p>
        <div className="mt-4 space-y-3">
          {actions.map((action) => (
            <div key={action} className="flex gap-3 text-sm/6">
              <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-(--app-active-text)" />
              <p className="text-(--app-text)">{action}</p>
            </div>
          ))}
        </div>
      </aside>
    </section>
  )
}
