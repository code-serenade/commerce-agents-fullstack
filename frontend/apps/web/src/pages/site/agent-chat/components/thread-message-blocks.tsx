import {
  BrainCircuit,
  Copy,
  CircleStop,
  RotateCcw,
  ListChecks,
  Loader2,
  Sparkles,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentChatStrategy } from "../domain/agent-chat-strategy"
import type { AgentChatMessage } from "../types"
import {
  getStudentNextActionMeta,
  getStudentNextActionPrompts,
} from "../view-model/student-next-actions-view-model"
import { AgentChatWelcomeContent } from "./agent-chat-welcome-content"

export function WelcomeCard({
  content,
  onSelectExamplePrompt,
}: {
  content: string
  onSelectExamplePrompt?: (prompt: string) => void
}) {
  const { t } = useTranslation()

  return (
    <div className="overflow-hidden rounded-[28px] border border-(--agent-chat-strong-border) bg-(--agent-chat-elevated-surface) bg-[image:var(--agent-chat-card-background)] shadow-[var(--agent-chat-card-shadow)] ring-1 ring-(--agent-chat-soft-border) backdrop-blur-xl">
      <div className="bg-[image:var(--agent-chat-card-header)] px-6 py-5">
        <div className="flex items-start gap-4">
          <div className="flex size-11 shrink-0 items-center justify-center rounded-2xl bg-(--agent-chat-primary-gradient) text-primary-foreground shadow-[var(--agent-chat-button-shadow)]">
            <BrainCircuit className="size-5" />
          </div>
          <div className="min-w-0">
            <p className="text-lg font-semibold text-foreground">
              {t("agentChat.thread.agentName")}
            </p>
            <p className="mt-1 text-sm/6 text-muted-foreground">{content}</p>
          </div>
        </div>
      </div>
      <div className="px-6 py-5">
        <AgentChatWelcomeContent
          onSelectExamplePrompt={onSelectExamplePrompt}
        />
      </div>
    </div>
  )
}

export function AgentProgressCard({ content }: { content?: string }) {
  const { t } = useTranslation()

  return (
    <div className="mt-4 flex items-center gap-2 rounded-2xl border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface) px-3 py-2 text-xs text-muted-foreground shadow-sm">
      <Loader2 className="size-4 animate-spin" />
      <span>
        {content
          ? t("agentChat.thread.receivingOutput")
          : t("agentChat.thread.waitingEvents")}
      </span>
    </div>
  )
}

export function AgentStoppedCard() {
  const { t } = useTranslation()

  return (
    <div className="mt-4 flex items-center gap-2 rounded-2xl border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface) px-3 py-2 text-xs text-muted-foreground shadow-sm">
      <CircleStop className="size-4" />
      <span>{t("agentChat.thread.stopped")}</span>
    </div>
  )
}

export function AgentErrorActions({
  fallbackPrompt,
  onSelectPrompt,
}: {
  fallbackPrompt?: string
  onSelectPrompt?: (prompt: string) => void
}) {
  const { t } = useTranslation()
  const retryPrompt =
    fallbackPrompt?.trim() || t("agentChat.thread.errorActions.retryPrompt")
  const shorterPrompt = t("agentChat.thread.errorActions.shorterPrompt")

  return (
    <div className="mt-3 rounded-lg border border-destructive/25 bg-destructive/5 p-3 text-xs text-destructive">
      <div className="flex min-w-0 items-center gap-2 font-medium">
        <CircleStop className="size-3.5 shrink-0" />
        <span>{t("agentChat.thread.errorActions.title")}</span>
      </div>
      <div className="mt-2 flex flex-wrap gap-2">
        <button
          type="button"
          onClick={() => onSelectPrompt?.(retryPrompt)}
          className="inline-flex h-8 items-center gap-1.5 rounded-md border border-destructive/25 bg-background px-3 text-[11px] font-medium transition hover:bg-destructive/10"
        >
          <RotateCcw className="size-3" />
          {t("agentChat.thread.errorActions.retry")}
        </button>
        <button
          type="button"
          onClick={() => onSelectPrompt?.(shorterPrompt)}
          className="inline-flex h-8 items-center gap-1.5 rounded-md border border-destructive/25 bg-background px-3 text-[11px] font-medium transition hover:bg-destructive/10"
        >
          <Sparkles className="size-3" />
          {t("agentChat.thread.errorActions.simplify")}
        </button>
      </div>
    </div>
  )
}

export function StrategyStatusPill({
  strategy,
}: {
  strategy: AgentChatStrategy
}) {
  const { t } = useTranslation()

  return (
    <div className="inline-flex max-w-full items-center gap-2 rounded-full border border-border bg-card px-3 py-1.5 text-xs text-muted-foreground shadow-sm">
      <BrainCircuit className="size-3.5 shrink-0 text-primary" />
      <span className="shrink-0 font-medium text-foreground">
        {t("agentChat.strategy.current")}
      </span>
      <span className="min-w-0 truncate">{t(strategy.labelKey)}</span>
    </div>
  )
}

export function MessageActions({ content }: { content: string }) {
  const { t } = useTranslation()
  const copyAnswer = () => {
    void navigator.clipboard?.writeText(content)
  }

  return (
    <div className="mt-3 flex items-center gap-1 text-muted-foreground">
      <button
        type="button"
        className="flex size-7 items-center justify-center rounded-md transition-colors hover:bg-muted hover:text-foreground"
        title={t("agentChat.thread.actions.copy")}
        onClick={copyAnswer}
      >
        <Copy className="size-4" />
      </button>
    </div>
  )
}

export function StudentNextActions({
  message,
  onSelectPrompt,
  previousUserContent,
}: {
  message?: AgentChatMessage
  onSelectPrompt?: (prompt: string) => void
  previousUserContent?: string
}) {
  const { t } = useTranslation()
  const prompts = getStudentNextActionPrompts(t, message, previousUserContent)
  const meta = getStudentNextActionMeta(t, message, previousUserContent)
  const showMetaLabel = meta.group !== "productFollowup"

  return (
    <div className="mt-3 w-full">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div className="min-w-0">
          <div className="flex items-center gap-2 text-xs font-semibold text-foreground">
            <ListChecks className="size-3.5 shrink-0 text-primary" />
            <span>{t("agentChat.thread.nextActions.title")}</span>
          </div>
          <p className="mt-1 text-[11px]/5 text-muted-foreground">
            {meta.detail}
          </p>
        </div>
        {showMetaLabel ? (
          <span className="shrink-0 rounded-full bg-(--agent-chat-accent-soft) px-2 py-1 text-[11px] font-medium text-(--agent-chat-accent-foreground)">
            {meta.label}
          </span>
        ) : null}
      </div>
      <div className="mt-2 flex flex-col gap-2">
        {prompts.map((prompt) => (
          <button
            key={prompt}
            type="button"
            onClick={() => onSelectPrompt?.(prompt)}
            className="group flex w-fit max-w-full min-w-0 cursor-pointer items-start rounded-lg bg-(--agent-chat-suggestion-background) px-2.5 py-1.5 text-left text-xs/5 text-(--agent-chat-suggestion-foreground) transition-colors hover:bg-(--agent-chat-suggestion-hover-background) hover:text-(--agent-chat-suggestion-foreground)"
          >
            <span className="min-w-0 flex-1">{prompt}</span>
          </button>
        ))}
      </div>
    </div>
  )
}
