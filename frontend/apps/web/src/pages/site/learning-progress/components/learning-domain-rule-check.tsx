import type { TFunction } from "i18next"
import { CheckCircle2, CircleDashed } from "lucide-react"
import type { LearningDomainAssessment } from "@/api"
import { getLearningEvaluationConditions } from "../domain/learning-evaluation-rules"

export function LearningDomainRuleCheck({
  assessment,
  t,
}: {
  assessment: LearningDomainAssessment
  t: TFunction
}) {
  const evaluationConditions = getLearningEvaluationConditions(assessment)

  return (
    <div>
      <p className="text-xs font-semibold text-foreground">
        {t("learningProgress.domainRuleCheck.title")}
      </p>
      <p className="mt-1 text-xs/5 text-muted-foreground">
        {t("learningProgress.domainRuleCheck.description")}
      </p>
      <ul className="mt-2 divide-y divide-(--agent-chat-soft-border)">
        {evaluationConditions.map((condition) => {
          const Icon = condition.satisfied ? CheckCircle2 : CircleDashed
          return (
            <li
              key={condition.code}
              className="flex items-start justify-between gap-3 py-2 first:pt-0 last:pb-0"
            >
              <span className="flex min-w-0 items-start gap-2">
                <Icon
                  aria-hidden="true"
                  className={
                    condition.satisfied
                      ? "mt-0.5 size-4 shrink-0 text-primary"
                      : "mt-0.5 size-4 shrink-0 text-muted-foreground"
                  }
                />
                <span>
                  <span className="block text-xs font-medium text-foreground">
                    {t(
                      `learningProgress.domainRuleCheck.conditions.${condition.code}.title`
                    )}
                  </span>
                  <span className="mt-0.5 block text-[11px]/4 text-muted-foreground">
                    {t(
                      `learningProgress.domainRuleCheck.conditions.${condition.code}.description`
                    )}
                  </span>
                </span>
              </span>
              <span className="shrink-0 text-[11px] text-muted-foreground">
                {condition.code === "structured_assessment"
                  ? t("learningProgress.domainRuleCheck.answersRecorded", {
                      current: condition.value,
                    })
                  : condition.code === "multiple_angles"
                    ? t("learningProgress.domainRuleCheck.angleProgress", {
                        current: condition.value,
                        target: condition.target,
                      })
                    : t(
                        condition.satisfied
                          ? "learningProgress.domainRuleCheck.met"
                          : "learningProgress.domainRuleCheck.notMet"
                      )}
              </span>
            </li>
          )
        })}
      </ul>
    </div>
  )
}
