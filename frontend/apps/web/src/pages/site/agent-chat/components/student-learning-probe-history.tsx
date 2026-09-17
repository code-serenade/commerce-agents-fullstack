import { BookOpenCheck } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { CompletedLearningProbe } from "../view-model/student-learning-probe-history-view-model"
import { StudentLearningProbeExplanation } from "./student-learning-probe-explanation"
import { StudentLearningProbeOutcome } from "./student-learning-probe-outcome"

export function StudentLearningProbeHistory({
  items,
}: {
  items: CompletedLearningProbe[]
}) {
  const { t } = useTranslation()

  if (!items.length) return null

  return (
    <section
      aria-label={t("agentChat.thread.learningProbe.historyTitle")}
      className="mt-3 overflow-hidden rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface)"
      data-learning-probe-history="true"
    >
      <div className="flex items-center gap-2 px-3.5 py-3 text-xs font-semibold text-foreground">
        <BookOpenCheck className="size-3.5 shrink-0 text-primary" />
        <span>
          {t("agentChat.thread.learningProbe.historyTitleWithCount", {
            count: items.length,
          })}
        </span>
      </div>
      <div className="border-t border-(--agent-chat-soft-border)">
        {items.map(({ probe, result }, index) => (
          <details
            className="border-b border-(--agent-chat-soft-border) px-3.5 py-2.5 last:border-b-0"
            key={probe.id}
          >
            <summary className="cursor-pointer text-xs/5 text-foreground marker:text-primary">
              <span className="ml-1 font-medium">
                {t("agentChat.thread.learningProbe.historyItem", {
                  number: index + 1,
                })}
              </span>
              <span className="ml-2 text-[11px]">
                <StudentLearningProbeOutcome
                  outcome={result.outcome ?? "unscorable"}
                />
              </span>
              <span className="mt-1 block text-[11px]/5 text-muted-foreground">
                {probe.questionText}
              </span>
            </summary>
            {result.outcome ? (
              <div className="mt-2 border-t border-(--agent-chat-soft-border) pt-2">
                <StudentLearningProbeExplanation
                  outcome={result.outcome}
                  result={result}
                />
              </div>
            ) : null}
          </details>
        ))}
      </div>
    </section>
  )
}
