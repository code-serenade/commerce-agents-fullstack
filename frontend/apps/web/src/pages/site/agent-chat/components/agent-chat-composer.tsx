import {
  BadgeDollarSign,
  BarChart3,
  Bot,
  ClipboardList,
  LockKeyhole,
  Loader2,
  Plus,
  PenLine,
  Rocket,
  Send,
  ShieldAlert,
  Square,
  type LucideIcon,
} from "lucide-react"
import { type KeyboardEvent, type MouseEvent, useRef } from "react"
import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components/stable/button"
import type { AgentChatProductContextSelection } from "../domain/agent-chat-context-selection"
import type { AgentChatPlaybookSelection } from "../domain/agent-chat-playbook"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import { AgentChatProductContextBar } from "./agent-chat-product-context-bar"
import { AgentChatRuntimeOptionsBar } from "./agent-chat-runtime-options-bar"
import { AgentChatStrategyBar } from "./agent-chat-strategy-bar"

const PLAYBOOK_ICONS: Record<string, LucideIcon> = {
  briefing: ClipboardList,
  creative: PenLine,
  launch: Rocket,
  pricing: BadgeDollarSign,
  review: BarChart3,
  service: ShieldAlert,
}

export interface AgentChatComposerProps {
  availableProducts: AgentChatProductContextSelection[]
  canSend: boolean
  input: string
  isLoggedIn: boolean
  isLoadingPlaybooks: boolean
  isLoadingProducts: boolean
  isLoadingRuntimeOptions: boolean
  isStreaming: boolean
  onInputChange: (value: string) => void
  onRuntimeCollaborationModeChange: (value: string) => void
  onRuntimeLearningLevelChange: (value: string) => void
  onRuntimeModelChange: (value: string) => void
  onRuntimePrimaryRoleChange: (value: string) => void
  onToggleRuntimeRole: (value: string) => void
  onSelectPlaybook: (playbook: AgentChatPlaybookSelection) => void
  onSend: () => void
  onStop: () => void
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  selectedProducts: AgentChatProductContextSelection[]
  playbooks: AgentChatPlaybookSelection[]
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  strategyId: AgentChatStrategyId
}

export function AgentChatComposer({
  availableProducts,
  canSend,
  input,
  isLoggedIn,
  isLoadingPlaybooks,
  isLoadingProducts,
  isLoadingRuntimeOptions,
  isStreaming,
  onInputChange,
  onRuntimeCollaborationModeChange,
  onRuntimeLearningLevelChange,
  onRuntimeModelChange,
  onRuntimePrimaryRoleChange,
  onToggleRuntimeRole,
  onSelectPlaybook,
  onSend,
  onStop,
  onStrategyChange,
  onToggleProductContext,
  selectedProducts,
  playbooks,
  runtimeOptions,
  runtimeSelection,
  strategyId,
}: AgentChatComposerProps) {
  const { t } = useTranslation()
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const appendComposerText = (text: string) => {
    onInputChange(input.trim() ? `${input.trim()}\n${text}` : text)
    queueMicrotask(() => textareaRef.current?.focus())
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

  const focusComposer = (event: MouseEvent<HTMLDivElement>) => {
    if (isStreaming) return
    const target = event.target
    if (!(target instanceof HTMLElement)) return
    if (target.closest("button, textarea")) return
    textareaRef.current?.focus()
  }

  return (
    <footer className="relative z-30 shrink-0 bg-background/95 p-3 sm:px-5">
      <div
        className="mx-auto flex w-full max-w-3xl flex-col gap-3 rounded-[1.65rem] border border-border bg-[#f1f1ef] p-3 shadow-xl shadow-black/10 dark:bg-card"
        onMouseDown={focusComposer}
        onKeyDown={handleKeyDown}
      >
        <AgentChatStrategyBar
          disabled={isStreaming}
          onStrategyChange={onStrategyChange}
          strategyId={strategyId}
        />
        <AgentChatProductContextBar
          availableProducts={availableProducts}
          disabled={isStreaming}
          isLoadingProducts={isLoadingProducts}
          onToggleProductContext={onToggleProductContext}
          selectedProducts={selectedProducts}
        />
        <AgentChatRuntimeOptionsBar
          disabled={isStreaming}
          isLoadingRuntimeOptions={isLoadingRuntimeOptions}
          onCollaborationModeChange={onRuntimeCollaborationModeChange}
          onLearningLevelChange={onRuntimeLearningLevelChange}
          onModelChange={onRuntimeModelChange}
          onPrimaryRoleChange={onRuntimePrimaryRoleChange}
          onToggleRole={onToggleRuntimeRole}
          runtimeOptions={runtimeOptions}
          runtimeSelection={runtimeSelection}
        />
        <div className="flex flex-wrap gap-2">
          {isLoadingPlaybooks ? (
            <span className="flex h-7 items-center gap-2 rounded-full border border-border bg-background/70 px-3 text-xs text-muted-foreground">
              <Loader2 className="size-3.5 animate-spin" />
              {t("agentChat.composer.playbooks.loading")}
            </span>
          ) : null}
          {!isLoadingPlaybooks && playbooks.length === 0 ? (
            <span className="flex h-7 items-center rounded-full border border-border bg-background/70 px-3 text-xs text-muted-foreground">
              {t("agentChat.composer.playbooks.empty")}
            </span>
          ) : null}
          {playbooks.map((playbook) => {
            const Icon = PLAYBOOK_ICONS[playbook.category] ?? ClipboardList
            return (
              <button
                key={playbook.id}
                type="button"
                disabled={isStreaming}
                onClick={() => {
                  onSelectPlaybook(playbook)
                  queueMicrotask(() => textareaRef.current?.focus())
                }}
                className="flex max-w-44 items-center gap-2 rounded-full border border-border bg-background px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
                title={`${playbook.description} ${playbook.expectedOutputs.join(" / ")}`}
              >
                <Icon className="size-3.5" />
                <span className="truncate">{playbook.title}</span>
              </button>
            )
          })}
        </div>
        <div className="flex min-w-0 items-end justify-between gap-2 rounded-3xl bg-background p-2 shadow-sm">
          <div className="flex min-w-0 flex-1 items-end gap-2 overflow-hidden">
            <button
              type="button"
              className="flex size-10 shrink-0 items-center justify-center rounded-full border border-border bg-card text-muted-foreground transition-colors hover:text-foreground"
              onClick={() => appendComposerText(t("agentChat.composer.attachPrompt"))}
              title={t("agentChat.composer.attachTitle")}
            >
              <Plus className="size-5" />
            </button>
            <div className="flex min-w-0 flex-1 flex-col gap-1 rounded-2xl p-1">
              <div className="flex min-w-0 items-center gap-2">
                <button
                  type="button"
                  className="flex max-w-48 min-w-0 shrink-0 items-center gap-2 rounded-full border border-border bg-card px-3 py-2 text-sm text-foreground transition-colors hover:bg-muted"
                  title={t("agentChat.composer.agentTitle")}
                >
                  <Bot className="size-4 shrink-0 text-blue-500" />
                  <span className="truncate">{t("agentChat.composer.agentName")}</span>
                </button>
                <p className="hidden shrink-0 text-xs text-muted-foreground lg:block">
                  {isLoggedIn
                    ? t("agentChat.composer.hint")
                    : t("agentChat.loginRequired.inline")}
                </p>
              </div>
              <textarea
                ref={textareaRef}
                value={input}
                onChange={(event) => onInputChange(event.target.value)}
                placeholder={t(
                  !isLoggedIn
                    ? "agentChat.composer.loginPlaceholder"
                    : "agentChat.composer.placeholder"
                )}
                disabled={isStreaming}
                rows={1}
                className="max-h-32 min-h-7 w-full resize-none border-0 bg-transparent px-2 py-1 text-sm/6 outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-60"
              />
            </div>
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
