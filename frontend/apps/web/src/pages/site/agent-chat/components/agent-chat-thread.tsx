import { Bot, UserRound } from "lucide-react"
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef } from "react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { AgentChatMessage } from "../types"
import { MessageContextSummary } from "./message-context-summary"
import { AssistantAnswer } from "./assistant-answer"
import { AgentChatThreadTurn } from "./agent-chat-thread-turn"
import {
  createThreadScrollSnapshot,
  restoreThreadScroll,
  type ThreadScrollSnapshot,
} from "../view-model/thread-scroll-preservation"
import {
  AgentProgressCard,
  AgentStoppedCard,
  StrategyStatusPill,
  MessageActions,
  StudentNextActions,
  WelcomeCard,
} from "./thread-message-blocks"
import type { AgentChatStrategy } from "../domain/agent-chat-strategy"
import { shouldShowStudentNextActions } from "../view-model/student-next-actions-view-model"

export interface AgentChatThreadProps {
  activeProcessMessageId: string | null
  isStreaming: boolean
  messages: AgentChatMessage[]
  onCloseProcessPanel: () => void
  onExamplePromptSelect?: (prompt: string) => void
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
  onExamplePromptSelect,
  onOpenProcessPanel,
  runtimeEvents,
  selectedStrategy,
  showProcessPanelAction,
}: AgentChatThreadProps) {
  const endRef = useRef<HTMLDivElement>(null)
  const scrollRootRef = useRef<HTMLElement>(null)
  const followStreamingRef = useRef(true)
  const ignoreScrollUntilRef = useRef(0)
  const hasStreamingMessage = messages.some(
    (message) => message.status === "streaming"
  )
  const inlineIsStreaming = isStreaming || hasStreamingMessage
  const messageIdentityKey = useMemo(
    () => messages.map((message) => message.id).join("|"),
    [messages]
  )
  const lastMessage = messages.at(-1)
  const lastMessageContentLength = lastMessage?.content.length ?? 0
  const previousMessageIdentityRef = useRef("")
  const pendingScrollRestoreRef = useRef<ThreadScrollSnapshot | null>(null)

  const scrollToEnd = useCallback((behavior: ScrollBehavior) => {
    ignoreScrollUntilRef.current = Date.now() + 350
    requestAnimationFrame(() => {
      const root = scrollRootRef.current
      if (root) {
        root.scrollTo({ behavior, top: root.scrollHeight })
      } else {
        endRef.current?.scrollIntoView({ behavior, block: "end" })
      }
    })
  }, [])

  useEffect(() => {
    if (messageIdentityKey === previousMessageIdentityRef.current) return
    previousMessageIdentityRef.current = messageIdentityKey
    followStreamingRef.current = true
    scrollToEnd("auto")
  }, [messageIdentityKey, scrollToEnd])

  useEffect(() => {
    if (!lastMessage) return
    if (!inlineIsStreaming && lastMessage.role !== "assistant") return
    if (activeProcessMessageId) return
    if (!followStreamingRef.current) return
    scrollToEnd("smooth")
  }, [
    activeProcessMessageId,
    inlineIsStreaming,
    lastMessage,
    lastMessageContentLength,
    scrollToEnd,
  ])

  useLayoutEffect(() => {
    restoreThreadScroll(pendingScrollRestoreRef.current)
  }, [activeProcessMessageId])

  const preserveThreadScroll = useCallback((messageId: string | null, action: () => void) => {
    const root = scrollRootRef.current
    const snapshot = createThreadScrollSnapshot(root, messageId)
    pendingScrollRestoreRef.current = snapshot
    action()
    restoreThreadScroll(snapshot)
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

  const handleScroll = () => {
    if (Date.now() < ignoreScrollUntilRef.current) return
    followStreamingRef.current = isNearThreadBottom(scrollRootRef.current)
  }

  return (
    <section
      ref={scrollRootRef}
      data-agent-chat-thread-scroll-root="true"
      onScroll={handleScroll}
      className="min-h-0 flex-1 overflow-y-auto bg-background px-4 py-5 sm:px-6"
    >
      <div className="mx-auto flex w-full max-w-3xl flex-col gap-6">
        {messages.length <= 1 ? (
          <div className="flex justify-center">
            <StrategyStatusPill strategy={selectedStrategy} />
          </div>
        ) : null}
        {messages.map((message, index) => (
          <AgentChatThreadTurn
            key={message.id}
            activeProcessMessageId={activeProcessMessageId}
            index={index}
            inlineIsStreaming={inlineIsStreaming}
            message={message}
            messages={messages}
            onCloseProcessPanel={handleCloseProcessPanel}
            onOpenProcessPanel={handleOpenProcessPanel}
            renderMessage={(targetMessage) => (
              <ThreadMessage
                message={targetMessage}
                onExamplePromptSelect={onExamplePromptSelect}
              />
            )}
            runtimeEvents={runtimeEvents}
            showProcessPanelAction={showProcessPanelAction}
          />
        ))}
        <div ref={endRef} />
      </div>
    </section>
  )
}

function ThreadMessage({
  message,
  onExamplePromptSelect,
}: {
  message: AgentChatMessage
  onExamplePromptSelect?: (prompt: string) => void
}) {
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
            <WelcomeCard
              content={message.content}
              onSelectExamplePrompt={onExamplePromptSelect}
            />
          ) : message.content && isUser ? (
            <p className="wrap-break-word whitespace-pre-wrap">
              {message.content}
            </p>
          ) : message.content ? (
            <AssistantAnswer content={message.content} />
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
          {shouldShowStudentNextActions(message) ? (
            <StudentNextActions
              message={message}
              onSelectPrompt={onExamplePromptSelect}
            />
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

function isNearThreadBottom(root: HTMLElement | null) {
  if (!root) return true
  return root.scrollHeight - root.scrollTop - root.clientHeight < 96
}
