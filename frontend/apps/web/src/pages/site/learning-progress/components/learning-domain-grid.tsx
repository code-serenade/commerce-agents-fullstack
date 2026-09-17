import { useTranslation } from "react-i18next"
import type { TFunction } from "i18next"
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
  Badge,
} from "@workspace/ui-components"
import type { LearningDomainAssessment } from "@/api"
import { translateLearningCode } from "../domain/learning-code-label"
import { LearningDomainAssessmentDetails } from "./learning-domain-assessment-details"
import { LearningEvaluationMethod } from "./learning-evaluation-method"

interface LearningDomainView {
  code: string
  label: string
  observed: boolean
  evaluationStatus?: string
  assessment?: LearningDomainAssessment
}

export function LearningDomainGrid({
  domains,
  onOpenConversation,
}: {
  domains: LearningDomainView[]
  onOpenConversation: (conversationId: string) => void
}) {
  const { t } = useTranslation()
  const needsAttention = domains
    .filter((domain) => domain.assessment?.statusCode === "needs_attention")
    .map((domain) => domain.code)

  return (
    <section className="border-t border-(--agent-chat-soft-border)/60 pt-5">
      <h2 className="text-sm font-semibold text-foreground">
        {t("learningProgress.domainTitle")}
      </h2>
      <p className="mt-1 text-xs/5 text-muted-foreground">
        {t("learningProgress.domainDescription")}
      </p>
      <LearningEvaluationMethod />
      <Accordion type="multiple" defaultValue={needsAttention}>
        {domains.map((domain) => (
          <AccordionItem key={domain.code} value={domain.code}>
            <AccordionTrigger>
              <div className="flex min-w-0 flex-1 flex-col gap-1 pr-3 text-left sm:flex-row sm:items-center sm:justify-between">
                <span className="font-medium text-foreground">
                  {domain.label}
                </span>
                <span className="flex flex-wrap items-center gap-2">
                  {domain.assessment?.assessmentCount ? (
                    <span className="text-xs font-normal text-muted-foreground">
                      {t("learningProgress.domainAssessmentCount", {
                        count: domain.assessment.assessmentCount,
                      })}
                    </span>
                  ) : null}
                  <Badge
                    variant={
                      domain.assessment?.statusCode === "demonstrated" ||
                            domain.assessment?.statusCode === "retained"
                        ? "secondary"
                        : "outline"
                    }
                  >
                    {getDomainStatusLabel(t, domain)}
                  </Badge>
                </span>
              </div>
            </AccordionTrigger>
            <AccordionContent>
              <LearningDomainAssessmentDetails
                assessment={domain.assessment}
                observed={domain.observed}
                onOpenConversation={onOpenConversation}
                t={t}
              />
            </AccordionContent>
          </AccordionItem>
        ))}
      </Accordion>
    </section>
  )
}

function getDomainStatusLabel(t: TFunction, domain: LearningDomainView) {
  if (domain.assessment) {
    return translateLearningCode(
      t,
      "learningProgress.domainStatus",
      domain.assessment.statusCode
    )
  }
  if (domain.evaluationStatus) {
    const key = `learningProgress.domainStatus.${domain.evaluationStatus}`
    const label = t(key)
    if (label !== key) return label
  }
  return domain.observed
    ? t("learningProgress.domainStatus.observed")
    : t("learningProgress.domainStatus.unobserved")
}
