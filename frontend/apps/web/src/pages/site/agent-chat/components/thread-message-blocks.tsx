import {
  BrainCircuit,
  Copy,
  CircleStop,
  ListChecks,
  Loader2,
  Sparkles,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentChatStrategy } from "../domain/agent-chat-strategy"

export function WelcomeCard({
  content,
  onSelectExamplePrompt,
}: {
  content: string
  onSelectExamplePrompt?: (prompt: string) => void
}) {
  const { t } = useTranslation()
  const capabilities = t("agentChat.thread.welcomeCapabilities", {
    returnObjects: true,
  }) as string[]
  const examplePrompts = t("agentChat.thread.examplePrompts", {
    returnObjects: true,
  }) as string[]

  return (
    <div className="rounded-xl border border-border bg-card p-5 shadow-sm">
      <div className="flex items-start gap-3">
        <div className="flex size-10 shrink-0 items-center justify-center rounded-md bg-foreground text-background">
          <BrainCircuit className="size-5" />
        </div>
        <div className="min-w-0">
          <p className="text-base font-semibold">
            {t("agentChat.thread.agentName")}
          </p>
          <p className="mt-1 text-sm/6 text-muted-foreground">{content}</p>
        </div>
      </div>
      <div className="mt-4 grid gap-2 sm:grid-cols-3">
        {capabilities.map((item, index) => (
          <div
            key={`${index}-${item}`}
            className="flex items-center gap-2 rounded-md bg-muted px-3 py-2 text-xs text-muted-foreground"
          >
            <ListChecks className="size-3.5" />
            {item}
          </div>
        ))}
      </div>
      <div className="mt-5 space-y-2">
        <p className="text-xs font-medium text-muted-foreground">
          {t("agentChat.thread.startHere")}
        </p>
        <div className="grid gap-2">
          {examplePrompts.map((prompt) => (
            <button
              key={prompt}
              type="button"
              className="group flex min-h-10 items-center gap-2 rounded-md border border-border bg-background px-3 py-2 text-left text-xs/5 text-foreground transition hover:border-foreground/30 hover:bg-muted"
              onClick={() => onSelectExamplePrompt?.(prompt)}
            >
              <Sparkles className="size-3.5 shrink-0 text-blue-500" />
              <span className="min-w-0 flex-1">{prompt}</span>
            </button>
          ))}
        </div>
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
