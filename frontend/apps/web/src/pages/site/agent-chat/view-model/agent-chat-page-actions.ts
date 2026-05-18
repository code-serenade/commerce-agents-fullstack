import type { Dispatch, MutableRefObject, SetStateAction } from "react"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import { createSessionId } from "../domain/agent-chat-message-mapping"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import { DEFAULT_AGENT_CHAT_STRATEGY_ID } from "../domain/agent-chat-strategy"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
  AgentChatMessage,
} from "../types"

export interface AgentChatPageStateActions {
  abortRef: MutableRefObject<AbortController | null>
  createWelcomeMessage: () => AgentChatMessage
  sessionIdRef: MutableRefObject<string>
  setConversationId: Dispatch<SetStateAction<string | null>>
  setConversations: Dispatch<SetStateAction<AgentChatConversationItem[]>>
  setError: Dispatch<SetStateAction<string | null>>
  setInput: Dispatch<SetStateAction<string>>
  setIsStreaming: Dispatch<SetStateAction<boolean>>
  setMessages: Dispatch<SetStateAction<AgentChatMessage[]>>
  setRuntimeEvents: Dispatch<SetStateAction<AgentRuntimeEvent[]>>
  setAttachments?: Dispatch<SetStateAction<AgentChatAttachmentItem[]>>
  clearSelectedCampaigns?: () => void
  clearSelectedProducts?: () => void
  clearSelectedSources?: () => void
  resetRuntimeSelection?: () => void
  restoreStrategyId?: (strategyId: AgentChatStrategyId) => void
}

export function resetGuestAgentChatState(
  state: AgentChatPageStateActions,
  isCancelled: () => boolean
) {
  queueMicrotask(() => {
    if (isCancelled()) return
    state.setConversationId(null)
    state.setConversations([])
    state.sessionIdRef.current = createSessionId()
    state.setMessages([state.createWelcomeMessage()])
    state.setAttachments?.([])
    state.clearSelectedCampaigns?.()
    state.clearSelectedProducts?.()
    state.clearSelectedSources?.()
    resetAgentChatConversationContext(state)
  })
}

export function startNewAgentConversation(state: AgentChatPageStateActions) {
  state.abortRef.current?.abort()
  state.abortRef.current = null
  state.setConversationId(null)
  state.sessionIdRef.current = createSessionId()
  state.setInput("")
  state.setError(null)
  state.setIsStreaming(false)
  state.setRuntimeEvents([])
  state.setMessages([state.createWelcomeMessage()])
  state.setAttachments?.([])
  state.clearSelectedCampaigns?.()
  state.clearSelectedProducts?.()
  state.clearSelectedSources?.()
  resetAgentChatConversationContext(state)
}

export function resetAgentChatConversationContext(
  state: Pick<
    AgentChatPageStateActions,
    "resetRuntimeSelection" | "restoreStrategyId"
  >
) {
  state.restoreStrategyId?.(DEFAULT_AGENT_CHAT_STRATEGY_ID)
  state.resetRuntimeSelection?.()
}

export function stopAgentStreaming(state: AgentChatPageStateActions) {
  state.abortRef.current?.abort()
  state.abortRef.current = null
  state.setIsStreaming(false)
  state.setMessages((current) =>
    current.map((message) =>
      message.status === "streaming"
        ? { ...message, status: message.content ? undefined : "stopped" }
        : message
    )
  )
}
