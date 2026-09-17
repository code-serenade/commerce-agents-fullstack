import { ArrowRight, ShieldCheck } from "lucide-react"
import { useTranslation } from "react-i18next"

const stages = [
  "notAssessed",
  "collecting",
  "demonstrated",
  "retained",
] as const

export function LearningEvaluationMethod() {
  const { t } = useTranslation()

  return (
    <div className="mt-4 border-y border-(--agent-chat-soft-border)/70 py-4">
      <div className="flex items-start gap-2">
        <ShieldCheck className="mt-0.5 size-4 shrink-0 text-primary" />
        <div className="min-w-0">
          <h3 className="text-xs font-semibold text-foreground">
            {t("learningProgress.evaluationMethod.title")}
          </h3>
          <p className="mt-1 text-xs/5 text-muted-foreground">
            {t("learningProgress.evaluationMethod.description")}
          </p>
        </div>
      </div>

      <ol className="mt-3 grid gap-2 sm:grid-cols-[1fr_auto_1fr_auto_1fr_auto_1fr] sm:items-center">
        {stages.map((stage, index) => (
          <li key={stage} className="contents">
            <div className="min-w-0 border-l-2 border-primary/45 pl-3">
              <p className="text-xs font-medium text-foreground">
                {t(`learningProgress.evaluationMethod.stages.${stage}.title`)}
              </p>
              <p className="mt-0.5 text-[11px]/4 text-muted-foreground">
                {t(
                  `learningProgress.evaluationMethod.stages.${stage}.description`
                )}
              </p>
            </div>
            {index < stages.length - 1 ? (
              <ArrowRight
                aria-hidden="true"
                className="hidden size-3.5 text-muted-foreground sm:block"
              />
            ) : null}
          </li>
        ))}
      </ol>

      <p className="mt-3 text-[11px]/5 text-muted-foreground">
        {t("learningProgress.evaluationMethod.priorityRule")}
      </p>
    </div>
  )
}
