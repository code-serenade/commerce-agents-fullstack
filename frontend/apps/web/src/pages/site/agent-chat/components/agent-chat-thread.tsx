import { useCallback, useEffect, useLayoutEffect, useMemo, useRef } from "react"
import type { AgentRuntimeEvent } from "@/api"
import type { AgentChatMessage } from "../types"
import { AgentChatThreadTurn } from "./agent-chat-thread-turn"
import {
  createThreadScrollSnapshot,
  restoreThreadScroll,
  type ThreadScrollSnapshot,
} from "../view-model/thread-scroll-preservation"
import { findPreviousUserContent } from "../view-model/agent-process-visibility-view-model"
import { AgentChatThreadMessage } from "./agent-chat-thread-message"
import { StrategyStatusPill } from "./thread-message-blocks"
import type { AgentChatStrategy } from "../domain/agent-chat-strategy"

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
  const hasOnlyWelcomeMessage =
    messages.length === 1 && messages[0]?.id === "welcome"

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

  useLayoutEffect(() => {
    const root = scrollRootRef.current
    if (!root) return

    const syncThreadViewportHeight = () => {
      root.style.setProperty(
        "--agent-chat-thread-viewport-height",
        `${root.clientHeight}px`
      )
    }

    syncThreadViewportHeight()

    if (typeof ResizeObserver === "undefined") return

    const observer = new ResizeObserver(syncThreadViewportHeight)
    observer.observe(root)
    return () => observer.disconnect()
  }, [])

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
      className="no-scrollbar min-h-0 flex-1 overflow-x-hidden overflow-y-auto bg-transparent px-4 pt-20 pb-6 sm:px-7 sm:pt-24 sm:pb-7"
    >
      <div className="mx-auto flex w-full max-w-[48rem] flex-col gap-6">
        {messages.length <= 1 && !hasOnlyWelcomeMessage ? (
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
              <AgentChatThreadMessage
                errorRetryPrompt={findPreviousUserContent(messages, index)}
                message={targetMessage}
                onExamplePromptSelect={onExamplePromptSelect}
                previousUserContent={findPreviousUserContent(messages, index)}
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
