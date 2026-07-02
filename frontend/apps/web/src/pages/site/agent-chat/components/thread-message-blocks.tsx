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
import { Link } from "react-router"
import type { AgentChatStrategy } from "../domain/agent-chat-strategy"
import type { AgentChatMessage } from "../types"
import {
  getStudentNextActionMeta,
  getStudentNextActionPrompts,
  getStudentTaskEntries,
} from "../view-model/student-next-actions-view-model"
import { AgentRoleClusterHomePanel } from "./agent-role-cluster-home-panel"

export function WelcomeCard({
  content,
  onSelectExamplePrompt,
}: {
  content: string
  onSelectExamplePrompt?: (prompt: string) => void
}) {
  const { t } = useTranslation()
  const taskEntries = getStudentTaskEntries(t)

  return (
    <div className="rounded-xl border border-border bg-card p-4 shadow-sm sm:p-5">
      <div className="flex items-start gap-3">
        <div className="flex size-10 shrink-0 items-center justify-center rounded-md bg-foreground text-background">
          <BrainCircuit className="size-5" />
        </div>
        <div className="min-w-0">
          <p className="text-base font-semibold">
            {t("agentChat.thread.agentName")}
          </p>
          <p className="mt-1 text-sm/6 text-muted-foreground">
            {content}
          </p>
        </div>
      </div>
      <div className="mt-4">
        <AgentRoleClusterHomePanel />
      </div>
      <div className="mt-4">
        <div className="flex min-w-0 flex-wrap items-center justify-between gap-2">
          <p className="text-xs font-semibold text-foreground">
            {t("agentChat.thread.studentTaskEntries.title")}
          </p>
          <Link
            to="/question-ideas"
            className="inline-flex min-h-7 items-center gap-1.5 rounded-md border border-border bg-background px-2.5 text-[11px] font-medium text-foreground transition hover:border-foreground/30 hover:bg-muted"
          >
            <Sparkles className="size-3.5 shrink-0 text-blue-500" />
            {t("agentChat.thread.studentTaskEntries.openIdeas")}
          </Link>
        </div>
        <p className="mt-1 text-xs text-muted-foreground">
          {t("agentChat.thread.studentTaskEntries.subtitle")}
        </p>
      </div>
      <div className="mt-2 grid gap-2 sm:grid-cols-2">
        {taskEntries.map((entry) => (
          <button
            key={entry.title}
            type="button"
            className="group min-h-20 rounded-md border border-border bg-background p-3 text-left transition hover:border-foreground/30 hover:bg-muted"
            onClick={() => onSelectExamplePrompt?.(entry.prompt)}
          >
            <span className="flex items-center gap-2 text-xs font-semibold text-foreground">
              <Sparkles className="size-3.5 shrink-0 text-blue-500" />
              {entry.title}
            </span>
            <span className="mt-2 block text-[11px]/5 text-muted-foreground">
              {entry.detail}
            </span>
            <span className="mt-2 line-clamp-2 block text-[11px]/5 text-blue-600 group-hover:text-blue-700">
              {entry.prompt}
            </span>
          </button>
        ))}
      </div>
    </div>
  )
}

export function AgentProgressCard({ content }: { content?: string }) {
  const { t } = useTranslation()

  return (
    <div className="mt-4 flex items-center gap-2 rounded-lg border border-border bg-card px-3 py-2 text-xs text-muted-foreground">
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
    <div className="mt-4 flex items-center gap-2 rounded-lg border border-border bg-card px-3 py-2 text-xs text-muted-foreground">
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
      <BrainCircuit className="size-3.5 shrink-0 text-blue-500" />
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

  return (
    <div className="mt-3 rounded-lg border border-border bg-card p-3 shadow-sm">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div className="min-w-0">
          <div className="flex items-center gap-2 text-xs font-semibold text-foreground">
            <ListChecks className="size-3.5 shrink-0 text-blue-500" />
            <span>{t("agentChat.thread.nextActions.title")}</span>
          </div>
          <p className="mt-1 text-[11px]/5 text-muted-foreground">
            {meta.detail}
          </p>
        </div>
        <span className="shrink-0 rounded-full bg-blue-50 px-2 py-1 text-[11px] font-medium text-blue-700 dark:bg-blue-950/40 dark:text-blue-200">
          {meta.label}
        </span>
      </div>
      <div className="mt-2 grid gap-1.5 sm:grid-cols-2">
        {prompts.map((prompt) => (
          <button
            key={prompt}
            type="button"
            onClick={() => onSelectPrompt?.(prompt)}
            className="group flex min-h-10 min-w-0 items-center gap-2 rounded-md border border-border bg-background px-3 py-2 text-left text-xs/5 text-foreground transition hover:border-foreground/30 hover:bg-muted"
          >
            <Sparkles className="size-3.5 shrink-0 text-blue-500" />
            <span className="min-w-0 flex-1">{prompt}</span>
          </button>
        ))}
      </div>
    </div>
  )
}
