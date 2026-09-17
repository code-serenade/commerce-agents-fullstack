import {
  CircleCheck,
  CircleDashed,
  CircleHelp,
  RefreshCcw,
  type LucideIcon,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import type { LearningProbeOutcome } from "@/api"

const OUTCOME_PRESENTATION: Record<
  LearningProbeOutcome,
  { Icon: LucideIcon }
> = {
  correct: {
    Icon: CircleCheck,
  },
  partial: {
    Icon: CircleDashed,
  },
  incorrect: {
    Icon: RefreshCcw,
  },
  unscorable: {
    Icon: CircleHelp,
  },
  legacy_unscored: {
    Icon: CircleHelp,
  },
}

export function StudentLearningProbeOutcome({
  outcome,
}: {
  outcome: LearningProbeOutcome
}) {
  const { t } = useTranslation()
  const { Icon } = OUTCOME_PRESENTATION[outcome]

  return (
    <span
      aria-label={t(`agentChat.thread.learningProbe.outcome.${outcome}`)}
      className="inline-flex max-w-full items-center gap-1.5 rounded-sm border border-primary/20 bg-primary/5 px-1.5 py-0.5 text-primary"
      data-learning-probe-outcome={outcome}
    >
      <Icon aria-hidden="true" className="size-3.5 shrink-0" />
      <span className="font-semibold">
        {t(`agentChat.thread.learningProbe.outcomeStatus.${outcome}`)}
      </span>
      <span aria-hidden="true" className="text-primary/45">
        |
      </span>
      <span className="font-normal text-muted-foreground">
        {t(`agentChat.thread.learningProbe.outcomeDetail.${outcome}`)}
      </span>
    </span>
  )
}
