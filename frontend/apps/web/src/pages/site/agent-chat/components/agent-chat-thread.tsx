import { useCallback, useEffect, useLayoutEffect, useRef } from "react"
import type { AgentRuntimeEvent } from "@/api"
import type { AgentChatMessage } from "../types"
import { AgentChatThreadTurn } from "./agent-chat-thread-turn"
import {
  createThreadScrollSnapshot,
  restoreThreadScroll,
  type ThreadScrollSnapshot,
} from "../view-model/thread-scroll-preservation"
import { findPreviousUserContent } from "../view-model/agent-process-visibility-view-model"
import {
  getThreadMessageIdentityKey,
  getVisibleThreadMessageEntries,
} from "../view-model/agent-chat-thread-view-model"
import { useAgentChatThreadViewportHeight } from "../view-model/use-agent-chat-thread-viewport-height"
import { AgentChatThreadMessage } from "./agent-chat-thread-message"
import { StrategyStatusPill } from "./thread-message-blocks"
import type { AgentChatStrategy } from "../domain/agent-chat-strategy"
import { getCurrentLearningProbeMessageId } from "../view-model/student-next-actions-view-model"

export interface AgentChatThreadProps {
  activeProcessMessageId: string | null
  conversationId?: string
  isStreaming: boolean
  messages: AgentChatMessage[]
  onCloseProcessPanel: () => void
  onExamplePromptSelect?: (prompt: string) => void
  onNextActionPromptSubmit?: (prompt: string) => void
  onAnswerProbe?: (question: string) => void
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
  conversationId,
  isStreaming,
  messages,
  onCloseProcessPanel,
  onExamplePromptSelect,
  onNextActionPromptSubmit,
  onAnswerProbe,
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
  const messageIdentityKey = getThreadMessageIdentityKey(messages)
  const lastMessage = messages.at(-1)
  const lastMessageContentLength = lastMessage?.content.length ?? 0
  const previousMessageIdentityRef = useRef("")
  const pendingScrollRestoreRef = useRef<ThreadScrollSnapshot | null>(null)
  const hasOnlyWelcomeMessage =
    messages.length === 1 && messages[0]?.id === "welcome"
  const visibleMessages = getVisibleThreadMessageEntries(messages)
  const learningProbeMessageId = getCurrentLearningProbeMessageId(messages)
  useAgentChatThreadViewportHeight(scrollRootRef)

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

  const scrollToStart = useCallback(() => {
    ignoreScrollUntilRef.current = Date.now() + 350
    requestAnimationFrame(() => {
      scrollRootRef.current?.scrollTo({ behavior: "auto", top: 0 })
    })
  }, [])

  useEffect(() => {
    if (messageIdentityKey === previousMessageIdentityRef.current) return
    previousMessageIdentityRef.current = messageIdentityKey
    if (hasOnlyWelcomeMessage) {
      scrollToStart()
      return
    }
    followStreamingRef.current = true
    scrollToEnd("auto")
  }, [hasOnlyWelcomeMessage, messageIdentityKey, scrollToEnd, scrollToStart])

  useEffect(() => {
    if (hasOnlyWelcomeMessage) return
    if (!lastMessage) return
    if (!inlineIsStreaming && lastMessage.role !== "assistant") return
    if (activeProcessMessageId) return
    if (!followStreamingRef.current) return
    scrollToEnd("smooth")
  }, [
    activeProcessMessageId,
    hasOnlyWelcomeMessage,
    inlineIsStreaming,
    lastMessage,
    lastMessageContentLength,
    scrollToEnd,
  ])

  useLayoutEffect(() => {
    restoreThreadScroll(pendingScrollRestoreRef.current)
  }, [activeProcessMessageId])

  const preserveThreadScroll = useCallback(
    (messageId: string | null, action: () => void) => {
      const root = scrollRootRef.current
      const snapshot = createThreadScrollSnapshot(root, messageId)
      pendingScrollRestoreRef.current = snapshot
      action()
      restoreThreadScroll(snapshot)
    },
    []
  )

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
      className="agent-chat-scrollarea min-h-0 flex-1 overflow-x-hidden overflow-y-auto bg-transparent px-4 pt-4 pb-6 [scrollbar-gutter:stable] sm:px-7 sm:pt-5 sm:pb-7"
    >
      <div className="mx-auto flex w-full max-w-3xl flex-col gap-6">
        {messages.length <= 1 && !hasOnlyWelcomeMessage ? (
          <div className="flex justify-center">
            <StrategyStatusPill strategy={selectedStrategy} />
          </div>
        ) : null}
        {visibleMessages.map(({ index, message }) => (
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
              <AgentChatThreadMessage
                conversationId={conversationId}
                errorRetryPrompt={findPreviousUserContent(messages, index)}
                message={targetMessage}
                onAnswerProbe={onAnswerProbe}
                onExamplePromptSelect={onExamplePromptSelect}
                onNextActionPromptSubmit={onNextActionPromptSubmit}
                previousUserContent={findPreviousUserContent(messages, index)}
                showLearningProbe={Boolean(conversationId)}
                isCurrentLearningProbe={targetMessage.id === learningProbeMessageId}
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

function isNearThreadBottom(root: HTMLElement | null) {
  if (!root) return true
  return root.scrollHeight - root.scrollTop - root.clientHeight < 96
}
