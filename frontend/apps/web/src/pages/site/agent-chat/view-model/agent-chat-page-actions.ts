import type { Dispatch, MutableRefObject, SetStateAction } from "react"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import { createSessionId } from "../domain/agent-chat-message-mapping"
import type { AgentChatConversationItem, AgentChatMessage } from "../types"

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
}

export function stopAgentStreaming(state: AgentChatPageStateActions) {
  state.abortRef.current?.abort()
  state.abortRef.current = null
  state.setIsStreaming(false)
  state.setMessages((current) =>
    current.map((message) =>
      message.status === "streaming"
        ? { ...message, status: undefined }
        : message
    )
  )
}
