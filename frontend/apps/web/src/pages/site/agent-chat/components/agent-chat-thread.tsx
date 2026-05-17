import { Bot, UserRound } from "lucide-react"
import { useCallback, useEffect, useRef } from "react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { AgentChatMessage } from "../types"
import { InlineSwarmTaskCard } from "./inline-swarm-task-card"
import { MessageContextSummary } from "./message-context-summary"
import { isChatOnlyRuntime } from "../view-model/thread-inline-view-model"
import {
  AgentProgressCard,
  AgentStoppedCard,
  StrategyStatusPill,
  MessageActions,
  WelcomeCard,
} from "./thread-message-blocks"
import type { AgentChatStrategy } from "../domain/agent-chat-strategy"

export interface AgentChatThreadProps {
  activeProcessMessageId: string | null
  isStreaming: boolean
  messages: AgentChatMessage[]
  onCloseProcessPanel: () => void
  onOpenProcessPanel: (target: {
    anchorLabel: string
    messageId: string
    task: string
  }) => void
  runtimeEvents: AgentRuntimeEvent[]
  selectedStrategy: AgentChatStrategy
  showProcessPanelAction: boolean
}

export function AgentChatThread({
  activeProcessMessageId,
  isStreaming,
  messages,
  onCloseProcessPanel,
  onOpenProcessPanel,
  runtimeEvents,
  selectedStrategy,
  showProcessPanelAction,
}: AgentChatThreadProps) {
  const endRef = useRef<HTMLDivElement>(null)
  const scrollRootRef = useRef<HTMLElement>(null)
  const hasStreamingMessage = messages.some(
    (message) => message.status === "streaming"
  )
  const inlineIsStreaming = isStreaming || hasStreamingMessage

  useEffect(() => {
    if (!inlineIsStreaming) return
    if (activeProcessMessageId) return
    if (!isNearThreadBottom(scrollRootRef.current)) return
    endRef.current?.scrollIntoView({ behavior: "smooth", block: "end" })
  }, [messages, inlineIsStreaming, activeProcessMessageId])

  const preserveThreadScroll = useCallback((messageId: string | null, action: () => void) => {
    const root = scrollRootRef.current
    const anchor = messageId
      ? root?.querySelector<HTMLElement>(`[data-message-id="${messageId}"]`)
      : null
    const fallbackScrollTop = root?.scrollTop
    const anchorTop = anchor?.getBoundingClientRect().top
    const activeElement = document.activeElement
    const shouldRefocus =
      activeElement instanceof HTMLElement && Boolean(root?.contains(activeElement))
    action()
    if (!root || fallbackScrollTop === undefined) return

    const restore = () => {
      if (shouldRefocus) activeElement.focus({ preventScroll: true })
      if (anchor && anchorTop !== undefined) {
        const nextTop = anchor.getBoundingClientRect().top
        root.scrollTop += nextTop - anchorTop
      } else {
        root.scrollTop = fallbackScrollTop
      }
    }
    restore()
    requestAnimationFrame(() => {
      restore()
      requestAnimationFrame(restore)
    })
  }, [])

  const handleOpenProcessPanel = (target: {
    anchorLabel: string
    messageId: string
    task: string
  }) => {
    preserveThreadScroll(target.messageId, () => onOpenProcessPanel(target))
  }

  const handleCloseProcessPanel = () => {
    preserveThreadScroll(activeProcessMessageId, onCloseProcessPanel)
  }

  return (
    <section
      ref={scrollRootRef}
      className="min-h-0 flex-1 overflow-y-auto bg-background px-4 py-5 sm:px-6"
    >
      <div className="mx-auto flex w-full max-w-3xl flex-col gap-6">
        {messages.length <= 1 ? (
          <div className="flex justify-center">
            <StrategyStatusPill strategy={selectedStrategy} />
          </div>
        ) : null}
        {messages.map((message, index) => (
          <div
            key={message.id}
            data-message-id={message.id}
            className="min-w-0 space-y-3"
          >
            {shouldRenderInlineProcess(message) ? (
              <InlineSwarmTaskCard
                messageId={message.id}
                isStreaming={message.status === "streaming" && inlineIsStreaming}
                onCloseProcessPanel={handleCloseProcessPanel}
                onOpenProcessPanel={() =>
                  handleOpenProcessPanel({
                    anchorLabel: findTurnLabel(messages, index),
                    messageId: message.id,
                    task: findPreviousUserTask(messages, index),
                  })
                }
                processPanelOpen={activeProcessMessageId === message.id}
                runtimeEvents={message.runtimeEvents ?? runtimeEvents}
                showProcessPanelAction={showProcessPanelAction}
                task={findPreviousUserTask(messages, index)}
              />
            ) : null}
            <ThreadMessage message={message} />
          </div>
        ))}
        <div ref={endRef} />
      </div>
    </section>
  )
}

function ThreadMessage({ message }: { message: AgentChatMessage }) {
  const { t } = useTranslation()
  const isUser = message.role === "user"
  const isWelcome = message.id === "welcome" && !isUser

  return (
    <article className={`flex gap-3 ${isUser ? "justify-end" : "justify-start"}`}>
      {!isUser ? (
        <div className="flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-card text-foreground">
          <Bot className="size-5" />
        </div>
      ) : null}
      <div className="flex max-w-[min(43rem,88%)] min-w-0 flex-col items-end">
        <div
          className={`min-w-0 text-sm/6 ${
            isUser
              ? "rounded-2xl bg-foreground px-4 py-3 text-background"
              : "text-foreground"
          }`}
        >
          {isWelcome ? (
            <WelcomeCard content={message.content} />
          ) : message.content ? (
            <p className="wrap-break-word whitespace-pre-wrap">
              {message.content}
            </p>
          ) : (
            <p className="text-muted-foreground">
              {t("agentChat.streaming")}
            </p>
          )}
          {message.status === "streaming" && !message.content ? (
            <AgentProgressCard content={message.content} />
          ) : null}
          {message.status === "stopped" ? <AgentStoppedCard /> : null}
          {message.status === "error" ? (
            <p className="mt-2 text-xs font-medium text-destructive">
              {t("agentChat.errorBadge")}
            </p>
          ) : null}
          {!isUser && message.content && !isWelcome ? (
            <MessageActions content={message.content} />
          ) : null}
        </div>
        {isUser ? (
          <MessageContextSummary summary={message.contextSummary} />
        ) : null}
      </div>
      {isUser ? (
        <div className="flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-card text-muted-foreground">
          <UserRound className="size-5" />
        </div>
      ) : null}
    </article>
  )
}

function shouldRenderInlineProcess(message: AgentChatMessage) {
  if (message.role !== "assistant") return false
  const runtimeEvents = message.runtimeEvents ?? []
  return runtimeEvents.length > 0 && !isChatOnlyRuntime(runtimeEvents)
}

function findPreviousUserTask(messages: AgentChatMessage[], fromIndex: number) {
  for (let index = fromIndex - 1; index >= 0; index -= 1) {
    const message = messages[index]
    if (message?.role === "user") return message.content
  }
  return ""
}

function findTurnLabel(messages: AgentChatMessage[], fromIndex: number) {
  const turn = messages
    .slice(0, fromIndex + 1)
    .filter((message) => message.role === "assistant" && message.id !== "welcome")
    .length
  return `#${String(Math.max(turn, 1)).padStart(2, "0")}`
}

function isNearThreadBottom(root: HTMLElement | null) {
  if (!root) return true
  return root.scrollHeight - root.scrollTop - root.clientHeight < 96
}
