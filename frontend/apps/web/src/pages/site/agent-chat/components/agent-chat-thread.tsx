import { Bot, UserRound } from "lucide-react"
import { useEffect, useRef } from "react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { AgentChatMessage } from "../types"
import { InlineProcessStatusNode } from "./inline-process-status-node"
import { InlineSwarmTaskCard } from "./inline-swarm-task-card"
import {
  AgentProgressCard,
  MessageActions,
  WelcomeCard,
} from "./thread-message-blocks"

export interface AgentChatThreadProps {
  isStreaming: boolean
  messages: AgentChatMessage[]
  processPanelOpen: boolean
  runtimeEvents: AgentRuntimeEvent[]
}

export function AgentChatThread({
  isStreaming,
  messages,
  processPanelOpen,
  runtimeEvents,
}: AgentChatThreadProps) {
  const { t } = useTranslation()
  const endRef = useRef<HTMLDivElement>(null)
  const latestUserTask = messages
    .filter((message) => message.role === "user")
    .at(-1)?.content
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
        {messages.map((message) => {
          const isUser = message.role === "user"
          const isWelcome = message.id === "welcome" && !isUser

          return (
            <article
              key={message.id}
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
                {message.status === "streaming" ? (
                  <AgentProgressCard content={message.content} />
                ) : null}
                {message.status === "error" ? (
                  <p className="mt-2 text-xs font-medium text-destructive">
                    {t("agentChat.errorBadge")}
                  </p>
                ) : null}
                {!isUser && message.content && !isWelcome ? (
                  <MessageActions />
                ) : null}
              </div>
              {isUser ? (
                <div className="flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-card text-muted-foreground">
                  <UserRound className="size-5" />
                </div>
              ) : null}
            </article>
          )
        })}
        {latestUserTask ? (
          <>
            <InlineSwarmTaskCard
              isStreaming={inlineIsStreaming}
              runtimeEvents={runtimeEvents}
              task={latestUserTask}
            />
            {!processPanelOpen ? (
              <InlineProcessStatusNode
                isStreaming={inlineIsStreaming}
                runtimeEvents={runtimeEvents}
              />
            ) : null}
          </>
        ) : null}
        <div ref={endRef} />
      </div>
    </section>
  )
}
