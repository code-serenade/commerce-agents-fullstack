import { ExternalLink, Lightbulb, Scale, TrendingUp } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { LearningProbeResultResponse } from "@/api"

export function StudentLearningProbeExplanation({
  outcome,
  result,
}: {
  outcome: string
  result: LearningProbeResultResponse
}) {
  const { t } = useTranslation()

  return (
    <>
      {result.selectedChoiceText || result.correctChoiceText ? (
        <dl className="mt-2 grid gap-2 border-t border-(--agent-chat-soft-border) pt-2 sm:grid-cols-2">
          {result.selectedChoiceText ? (
            <div className="min-w-0">
              <dt className="text-[11px] font-medium text-muted-foreground">
                {t("agentChat.thread.learningProbe.selectedChoice")}
              </dt>
              <dd className="mt-1 text-xs/5 wrap-break-word text-foreground">
                {result.selectedChoiceText}
              </dd>
            </div>
          ) : null}
          {result.correctChoiceText ? (
            <div className="min-w-0 sm:border-l sm:border-(--agent-chat-soft-border) sm:pl-3">
              <dt className="text-[11px] font-medium text-primary">
                {t("agentChat.thread.learningProbe.correctChoice")}
              </dt>
              <dd className="mt-1 text-xs/5 font-medium wrap-break-word text-foreground">
                {result.correctChoiceText}
              </dd>
            </div>
          ) : null}
        </dl>
      ) : null}
      {result.answerComparisonCode ? (
        <div className="mt-2 flex items-start gap-2 text-[11px]/5 text-foreground">
          <Scale className="mt-0.5 size-3.5 shrink-0 text-primary" />
          <p>
            {t(
              `agentChat.thread.learningProbe.answerComparison.${result.answerComparisonCode}`
            )}
          </p>
        </div>
      ) : null}
      {result.referenceExplanation ? (
        <div className="mt-2 border-t border-(--agent-chat-soft-border) pt-2">
          <p className="text-[11px] font-medium text-foreground">
            {t("agentChat.thread.learningProbe.referenceAnswer")}
          </p>
          <p className="mt-1 text-[11px]/5 text-muted-foreground">
            {result.referenceExplanation}
          </p>
        </div>
      ) : null}
      {result.knowledgePoints?.length ? (
        <div className="mt-2 border-t border-(--agent-chat-soft-border) pt-2">
          <p className="text-[11px] font-medium text-foreground">
            {t("agentChat.thread.learningProbe.knowledgePoints")}
          </p>
          <ul className="mt-1 space-y-1 text-[11px]/5 text-muted-foreground">
            {result.knowledgePoints.map((point) => (
              <li key={point} className="flex items-start gap-2">
                <span className="mt-2 size-1 shrink-0 rounded-full bg-primary" />
                <span>{point}</span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
      {result.referenceExplanation || result.knowledgePoints?.length ? (
        <div className="mt-2 flex items-start gap-2 border-t border-(--agent-chat-soft-border) pt-2 text-[11px]/5 text-muted-foreground">
          <Lightbulb className="mt-0.5 size-3.5 shrink-0 text-primary" />
          <div>
            <p className="font-medium text-foreground">
              {t("agentChat.thread.learningProbe.reviewGuidanceTitle")}
            </p>
            <p className="mt-0.5">
              {t(
                `agentChat.thread.learningProbe.reviewGuidance.${reviewGuidanceKey(outcome)}`
              )}
            </p>
          </div>
        </div>
      ) : null}
      <div className="mt-2 flex items-start gap-2 border-t border-(--agent-chat-soft-border) pt-2 text-[11px]/5 text-muted-foreground">
        <TrendingUp className="mt-0.5 size-3.5 shrink-0 text-primary" />
        <div>
          <p className="font-medium text-foreground">
            {t("agentChat.thread.learningProbe.assessmentImpactTitle")}
          </p>
          <p className="mt-0.5">
            {t(
              `agentChat.thread.learningProbe.assessmentImpact.${assessmentImpactKey(outcome)}`
            )}
          </p>
        </div>
      </div>
      {result.sourceLabels.length ? (
        <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px]">
          <span className="text-muted-foreground">
            {t("agentChat.thread.learningProbe.sources")}
          </span>
          {result.sourceLabels.map((source) => (
            <a
              key={source.sourceId}
              className="inline-flex items-center gap-1 text-primary hover:underline"
              href={source.url}
              rel="noreferrer"
              target="_blank"
            >
              {source.publisher}
              <ExternalLink className="size-3" />
            </a>
          ))}
        </div>
      ) : null}
    </>
  )
}

function reviewGuidanceKey(outcome: string) {
  if (outcome === "incorrect") return "incorrect"
  if (outcome === "partial") return "partial"
  return "correct"
}

function assessmentImpactKey(outcome: string) {
  if (outcome === "correct") return "correct"
  if (outcome === "partial") return "partial"
  if (outcome === "incorrect") return "incorrect"
  return "unscored"
}
