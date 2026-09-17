import { useEffect, useRef, useState } from "react"
import { ChevronDown, ChevronUp, HelpCircle, Sparkles } from "lucide-react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router"
import { StudentLearningProbeActions } from "./student-learning-probe-actions"
import { StudentLearningProbeChoices } from "./student-learning-probe-choices"
import { StudentLearningProbeResult } from "./student-learning-probe-result"
import { useStudentLearningProbeVisibility } from "./use-student-learning-probe-visibility"
import type { LearningProbeResultResponse } from "@/api"

export interface StudentLearningProbeViewData {
  id: string
  questionText: string
  questionType?: "single_choice" | "open_response"
  choices?: Array<{ id: string; text: string }>
  interactionMode?: "contextual_micro"
  rewardCode?: "progress_feedback"
  engaged?: boolean
}

export function StudentLearningProbe({
  probe,
  onAnswer,
  onEngage,
  onSubmitChoice,
  onContinue,
  isCurrent = true,
  ready,
  result,
  resultFailed = false,
  onRetryResult,
  continuing = false,
  continueFailed = false,
  continueUnavailable = false,
  submittingChoice = false,
}: {
  probe: StudentLearningProbeViewData | null
  onAnswer: (question: string) => void
  onEngage: (probeId: string) => Promise<boolean>
  onSubmitChoice?: (probeId: string, choiceId: string) => Promise<boolean>
  onSkip?: (probeId: string) => Promise<boolean> | void
  onDismissResult?: () => void
  onContinue?: (probeId: string) => void
  isCurrent?: boolean
  ready: boolean
  result?: LearningProbeResultResponse | null
  resultFailed?: boolean
  onRetryResult?: () => void
  continuing?: boolean
  continueFailed?: boolean
  continueUnavailable?: boolean
  skipping?: boolean
  submittingChoice?: boolean
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [engaged, setEngaged] = useState(false)
  const [starting, setStarting] = useState(false)
  const [engageFailed, setEngageFailed] = useState(false)
  const [selectedChoiceId, setSelectedChoiceId] = useState<string | null>(null)
  const [choiceSubmitFailed, setChoiceSubmitFailed] = useState(false)
  const [collapsed, setCollapsed] = useState(!isCurrent)
  const cardRef = useRef<HTMLDivElement>(null)
  const engageInFlightRef = useRef(false)
  const choiceSubmitInFlightRef = useRef(false)

  useStudentLearningProbeVisibility({
    cardRef,
    probeId: probe?.id,
    ready,
    result,
  })

  useEffect(() => {
    setEngaged(Boolean(probe?.engaged))
    setStarting(false)
    setEngageFailed(false)
    setSelectedChoiceId(null)
    setChoiceSubmitFailed(false)
    choiceSubmitInFlightRef.current = false
    engageInFlightRef.current = false
  }, [probe?.id, probe?.engaged])

  useEffect(() => {
    setCollapsed(!isCurrent)
  }, [isCurrent])

  if (!probe || !ready) return null
  const isEngaged = engaged || probe.engaged || Boolean(result)

  const startAnswer = async () => {
    if (starting || engageInFlightRef.current) return
    engageInFlightRef.current = true
    setEngageFailed(false)
    setStarting(true)
    try {
      const engaged = await onEngage(probe.id)
      if (!engaged) {
        setEngageFailed(true)
        return
      }
      setEngaged(true)
      onAnswer(t("agentChat.thread.learningProbe.answerStarter"))
    } catch {
      setEngageFailed(true)
    } finally {
      setStarting(false)
      engageInFlightRef.current = false
    }
  }

  const isSingleChoice =
    probe.questionType === "single_choice" && Boolean(probe.choices?.length)
  const submitChoice = async () => {
    if (
      !selectedChoiceId ||
      !onSubmitChoice ||
      submittingChoice ||
      choiceSubmitInFlightRef.current
    ) {
      return
    }
    choiceSubmitInFlightRef.current = true
    setChoiceSubmitFailed(false)
    try {
      const submitted = await onSubmitChoice(probe.id, selectedChoiceId)
      setChoiceSubmitFailed(!submitted)
    } catch {
      setChoiceSubmitFailed(true)
    } finally {
      choiceSubmitInFlightRef.current = false
    }
  }
  const showResult = isEngaged
  const questionId = `learning-probe-question-${probe.id}`

  return (
    <div
      ref={cardRef}
      className="mt-3 w-full rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface) p-3.5"
    >
      <div className="flex items-center gap-2 text-xs font-semibold text-foreground">
        <span className="flex size-6 shrink-0 items-center justify-center rounded-md bg-(--agent-chat-accent-soft)">
          <HelpCircle className="size-3.5 text-primary" />
        </span>
        <span>{t("agentChat.thread.learningProbe.title")}</span>
        <span className="min-w-0 flex-1 truncate text-right text-[11px] font-normal text-muted-foreground">
          {result?.outcome
            ? t("agentChat.thread.learningProbe.compactRecorded")
            : t("agentChat.thread.learningProbe.compactPending")}
        </span>
        <button
          type="button"
          aria-expanded={!collapsed}
          aria-label={t(
            collapsed
              ? "agentChat.thread.learningProbe.expand"
              : "agentChat.thread.learningProbe.collapse"
          )}
          className="inline-flex size-7 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          onClick={() => setCollapsed((value) => !value)}
          title={t(
            collapsed
              ? "agentChat.thread.learningProbe.expand"
              : "agentChat.thread.learningProbe.collapse"
          )}
        >
          {collapsed ? (
            <ChevronDown className="size-3.5" />
          ) : (
            <ChevronUp className="size-3.5" />
          )}
        </button>
      </div>
      {collapsed ? (
        <button
          type="button"
          className="mt-2 w-full truncate border-l-2 border-primary pl-3 text-left text-xs/5 font-medium text-foreground hover:text-primary"
          onClick={() => setCollapsed(false)}
        >
          {probe.questionText}
        </button>
      ) : (
        <>
      {!isSingleChoice ? (
        <p className="mt-1.5 text-[11px]/4 text-muted-foreground">
          {t("agentChat.thread.learningProbe.contextHint")}
        </p>
      ) : null}
      <div className="mt-3 border-l-2 border-primary pl-3">
        <p className="text-[11px] font-medium text-primary">
          {t("agentChat.thread.learningProbe.questionLabel")}
        </p>
        <p
          id={questionId}
          className="mt-1 text-sm/6 font-semibold text-foreground"
        >
          {probe.questionText}
        </p>
      </div>
      {!isSingleChoice ? (
        <div className="mt-2 flex items-start gap-1.5 text-[11px]/5 text-muted-foreground">
          <Sparkles className="mt-0.5 size-3 shrink-0 text-primary" />
          <span>
            {t("agentChat.thread.learningProbe.reward.progress_feedback")}
          </span>
        </div>
      ) : null}
      {showResult ? (
        <StudentLearningProbeResult
          continuing={continuing}
          continueFailed={continueFailed}
          continueUnavailable={continueUnavailable}
          evaluationPending={isSingleChoice && !result}
          resultFailed={resultFailed}
          onRetryResult={onRetryResult}
          onContinue={onContinue ? () => onContinue(probe.id) : undefined}
          onViewProgress={() => navigate("/learning-progress")}
          result={result ?? null}
        />
      ) : (
        <div className="mt-3">
          {isSingleChoice ? (
            <StudentLearningProbeChoices
              ariaLabelledBy={questionId}
              choices={probe.choices ?? []}
              disabled={starting || submittingChoice}
              selectedChoiceId={selectedChoiceId}
              onSelect={(choiceId) => {
                setSelectedChoiceId(choiceId)
                setChoiceSubmitFailed(false)
              }}
            />
          ) : null}
          <StudentLearningProbeActions
            choiceSubmitFailed={choiceSubmitFailed}
            engageFailed={engageFailed}
            isSingleChoice={isSingleChoice}
            selectedChoiceId={selectedChoiceId}
            starting={starting}
            submittingChoice={submittingChoice}
            onStartAnswer={() => void startAnswer()}
            onSubmitChoice={() => void submitChoice()}
            onCollapse={() => setCollapsed(true)}
            onViewProgress={() => navigate("/learning-progress")}
          />
        </div>
      )}
        </>
      )}
    </div>
  )
}
