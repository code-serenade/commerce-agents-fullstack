import { Bot, UserRound } from "lucide-react"
import { Fragment, useEffect, useRef } from "react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { AgentChatMessage } from "../types"
import { InlineSwarmTaskCard } from "./inline-swarm-task-card"
import { isChatOnlyRuntime } from "../view-model/thread-inline-view-model"
import {
  AgentProgressCard,
  StrategyStatusPill,
  MessageActions,
  WelcomeCard,
} from "./thread-message-blocks"
import type { AgentChatStrategy } from "../domain/agent-chat-strategy"

export interface AgentChatThreadProps {
  isStreaming: boolean
  messages: AgentChatMessage[]
  onOpenProcessPanel: () => void
  processPanelOpen: boolean
  runtimeEvents: AgentRuntimeEvent[]
  selectedStrategy: AgentChatStrategy
  showProcessPanelAction: boolean
}

export function AgentChatThread({
  isStreaming,
  messages,
  onOpenProcessPanel,
  processPanelOpen,
  runtimeEvents,
  selectedStrategy,
  showProcessPanelAction,
}: AgentChatThreadProps) {
  const endRef = useRef<HTMLDivElement>(null)
  const hasStreamingMessage = messages.some(
    (message) => message.status === "streaming"
  )
  const inlineIsStreaming = isStreaming || hasStreamingMessage

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth", block: "end" })
  }, [messages, processPanelOpen])

  return (
    <section className="min-h-0 flex-1 overflow-y-auto bg-background px-4 py-5 sm:px-6">
      <div className="mx-auto flex w-full max-w-3xl flex-col gap-6">
        {messages.length <= 1 ? (
          <div className="flex justify-center">
            <StrategyStatusPill strategy={selectedStrategy} />
          </div>
        ) : null}
        {messages.map((message, index) => (
          <Fragment key={message.id}>
            {shouldRenderInlineProcess(message) ? (
              <InlineSwarmTaskCard
                key={`process-${message.id}`}
                isStreaming={message.status === "streaming" && inlineIsStreaming}
                onOpenProcessPanel={onOpenProcessPanel}
                processPanelOpen={processPanelOpen}
                runtimeEvents={message.runtimeEvents ?? runtimeEvents}
                showProcessPanelAction={showProcessPanelAction}
                task={findPreviousUserTask(messages, index)}
              />
            ) : null}
            <ThreadMessage
              message={message}
            />
          </Fragment>
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
    <article
      className={`flex gap-3 ${isUser ? "justify-end" : "justify-start"}`}
    >
      {!isUser ? (
        <div className="flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-card text-foreground">
          <Bot className="size-5" />
        </div>
      ) : null}
      <div
        className={`max-w-[min(43rem,88%)] min-w-0 text-sm/6 ${
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
