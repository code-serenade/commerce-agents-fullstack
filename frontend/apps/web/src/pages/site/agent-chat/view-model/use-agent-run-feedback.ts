import { useCallback, useEffect, useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "@workspace/ui-components"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import { agentChatData } from "../agent-chat-data"
import {
  getAgentRunFeedbackTarget,
  type AgentRunFeedbackTarget,
} from "./agent-run-feedback-view-model"

export interface AgentRunFeedbackState {
  canSubmit: boolean
  comment: string
  disabledReason: string | null
  error: string | null
  isSubmitting: boolean
  rating: number
  setComment: (comment: string) => void
  setRating: (rating: number) => void
  submitted: boolean
  submit: () => Promise<void>
  target: AgentRunFeedbackTarget | null
}

export function useAgentRunFeedback(input: {
  isLoggedIn: boolean
  isStreaming: boolean
  runtimeEvents: AgentRuntimeEvent[]
}): AgentRunFeedbackState {
  const { t } = useTranslation()
  const target = useMemo(
    () => getAgentRunFeedbackTarget(input.runtimeEvents),
    [input.runtimeEvents]
  )
  const [rating, setRating] = useState(5)
  const [comment, setComment] = useState("")
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submittedRunIds, setSubmittedRunIds] = useState<Set<string>>(
    () => new Set()
  )
  const [error, setError] = useState<string | null>(null)
  const submitted = target ? submittedRunIds.has(target.runId) : false

  useEffect(() => {
    setError(null)
    setComment("")
    setRating(5)
  }, [target?.runId])

  const disabledReason = useMemo(() => {
    if (!input.isLoggedIn) return t("agentChat.feedback.disabled.login")
    if (!target) return t("agentChat.feedback.disabled.noRun")
    if (input.isStreaming) return t("agentChat.feedback.disabled.streaming")
    if (submitted) return t("agentChat.feedback.disabled.submitted")
    return null
  }, [input.isLoggedIn, input.isStreaming, submitted, target, t])

  const submit = useCallback(async () => {
    if (!target || disabledReason || isSubmitting) return

    setIsSubmitting(true)
    setError(null)
    try {
      await agentChatData.submitRunFeedback(target.workspaceId, target.runId, {
        rating,
        comment: comment.trim() || null,
        metadata: {
          source: "agent_chat",
          workspaceTitle: target.workspaceTitle ?? null,
          runStatus: target.runStatus,
        },
      })
      setSubmittedRunIds((current) => new Set(current).add(target.runId))
      toast.success(t("agentChat.feedback.notify.success"))
    } catch (err) {
      const message =
        err instanceof Error ? err.message : t("agentChat.feedback.notify.failed")
      setError(message)
      toast.error(t("agentChat.feedback.notify.failed"))
    } finally {
      setIsSubmitting(false)
    }
  }, [comment, disabledReason, isSubmitting, rating, target, t])

  return {
    canSubmit: !disabledReason && !isSubmitting,
    comment,
    disabledReason,
    error,
    isSubmitting,
    rating,
    setComment,
    setRating,
    submitted,
    submit,
    target,
  }
}
