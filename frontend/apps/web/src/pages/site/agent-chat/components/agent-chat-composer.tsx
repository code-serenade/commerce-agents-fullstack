import {
  Bot,
  Coffee,
  Database,
  LockKeyhole,
  Loader2,
  Megaphone,
  Plus,
  Send,
  ShoppingBag,
  Square,
} from "lucide-react"
import { type KeyboardEvent } from "react"
import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components/stable/button"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import { AgentChatStrategyBar } from "./agent-chat-strategy-bar"

const SAMPLE_PROMPTS = [
  {
    labelKey: "agentChat.composer.samples.coffee.label",
    icon: Coffee,
    promptKey: "agentChat.composer.samples.coffee.prompt",
  },
  {
    labelKey: "agentChat.composer.samples.qianchuan.label",
    icon: Megaphone,
    promptKey: "agentChat.composer.samples.qianchuan.prompt",
  },
]

const CONTEXT_CHIPS = [
  {
    labelKey: "agentChat.composer.chips.products.label",
    icon: ShoppingBag,
    titleKey: "agentChat.composer.chips.products.title",
    promptKey: "agentChat.composer.chips.products.prompt",
  },
  {
    labelKey: "agentChat.composer.chips.campaign.label",
    icon: Megaphone,
    titleKey: "agentChat.composer.chips.campaign.title",
    promptKey: "agentChat.composer.chips.campaign.prompt",
  },
  {
    labelKey: "agentChat.composer.chips.platform.label",
    icon: Database,
    titleKey: "agentChat.composer.chips.platform.title",
    promptKey: "agentChat.composer.chips.platform.prompt",
  },
]

export interface AgentChatComposerProps {
  canSend: boolean
  input: string
  isLoggedIn: boolean
  isStreaming: boolean
  onInputChange: (value: string) => void
  onSend: () => void
  onStop: () => void
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  strategyId: AgentChatStrategyId
}

export function AgentChatComposer({
  canSend,
  input,
  isLoggedIn,
  isStreaming,
  onInputChange,
  onSend,
  onStop,
  onStrategyChange,
  strategyId,
}: AgentChatComposerProps) {
  const { t } = useTranslation()

  const appendComposerText = (text: string) => {
    onInputChange(input.trim() ? `${input.trim()}\n${text}` : text)
  }

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (
      event.key !== "Enter" ||
      event.shiftKey ||
      event.nativeEvent.isComposing
    )
      return
    event.preventDefault()
    if (canSend) onSend()
  }

  return (
    <footer className="shrink-0 bg-background/95 p-3 sm:px-5">
      <div
        className="mx-auto flex w-full max-w-3xl flex-col gap-3 rounded-[1.65rem] border border-border bg-[#f1f1ef] p-3 shadow-xl shadow-black/10 dark:bg-card"
        onKeyDown={handleKeyDown}
      >
        <AgentChatStrategyBar
          disabled={isStreaming}
          onStrategyChange={onStrategyChange}
          strategyId={strategyId}
        />
        <div className="flex flex-wrap gap-2">
          {SAMPLE_PROMPTS.map((sample) => {
            const Icon = sample.icon
            const label = t(sample.labelKey)
            const prompt = t(sample.promptKey)
            return (
              <button
                key={sample.labelKey}
                type="button"
                disabled={isStreaming}
                onClick={() => onInputChange(prompt)}
                className="flex items-center gap-2 rounded-full border border-border bg-background px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
                title={prompt}
              >
                <Icon className="size-3.5" />
                {label}
              </button>
            )
          })}
          <span className="hidden h-7 w-px bg-border sm:block" />
          {CONTEXT_CHIPS.map((chip) => {
            const Icon = chip.icon
            const label = t(chip.labelKey)
            const prompt = t(chip.promptKey)
            return (
              <button
                key={chip.labelKey}
                type="button"
                disabled={isStreaming}
                onClick={() => appendComposerText(prompt)}
                className="flex items-center gap-2 rounded-full border border-border bg-background/70 px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
                title={`${t(chip.titleKey)} ${prompt}`}
              >
                <Icon className="size-3.5" />
                {label}
              </button>
            )
          })}
        </div>
        <textarea
          value={input}
          onChange={(event) => onInputChange(event.target.value)}
          placeholder={t(
            !isLoggedIn
              ? "agentChat.composer.loginPlaceholder"
              : "agentChat.composer.placeholder"
          )}
          disabled={isStreaming}
          rows={2}
          className="max-h-32 min-h-14 w-full resize-none rounded-md border-0 bg-transparent p-2 text-sm/6 outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-60"
        />
        <div className="flex min-w-0 items-center justify-between gap-2 rounded-full bg-background p-2 shadow-sm">
          <div className="flex min-w-0 flex-1 items-center gap-2 overflow-hidden">
            <button
              type="button"
              className="flex size-10 shrink-0 items-center justify-center rounded-full border border-border bg-card text-muted-foreground transition-colors hover:text-foreground"
              onClick={() => appendComposerText(t("agentChat.composer.attachPrompt"))}
              title={t("agentChat.composer.attachTitle")}
            >
              <Plus className="size-5" />
            </button>
            <button
              type="button"
              className="flex max-w-56 min-w-0 shrink items-center gap-2 rounded-full border border-border bg-card px-3 py-2 text-sm text-foreground transition-colors hover:bg-muted"
              title={t("agentChat.composer.agentTitle")}
            >
              <Bot className="size-4 shrink-0 text-blue-500" />
              <span className="truncate">{t("agentChat.composer.agentName")}</span>
            </button>
            <p className="hidden shrink-0 text-xs text-muted-foreground min-[1700px]:block">
              {isLoggedIn
                ? t("agentChat.composer.hint")
                : t("agentChat.loginRequired.inline")}
            </p>
          </div>
          {isStreaming ? (
            <Button variant="outline" size="icon" onClick={onStop}>
              <Square className="size-4" />
            </Button>
          ) : (
            <button
              type="button"
              disabled={!canSend}
              onClick={onSend}
              className="flex size-9 shrink-0 items-center justify-center rounded-full bg-foreground text-background transition-colors hover:bg-foreground/90 disabled:cursor-not-allowed disabled:opacity-45"
              title={t("agentChat.composer.send")}
            >
              {canSend ? (
                isLoggedIn ? (
                  <Send className="size-4" />
                ) : (
                  <LockKeyhole className="size-4" />
                )
              ) : (
                <Loader2 className="size-4" />
              )}
            </button>
          )}
        </div>
      </div>
    </footer>
  )
}
