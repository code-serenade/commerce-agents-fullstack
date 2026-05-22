import { useCallback, type MutableRefObject } from "react"
import type { Dispatch, SetStateAction } from "react"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import type {
  AgentChatCampaignContextSelection,
  AgentChatProductContextSelection,
  AgentChatSourceContextSelection,
} from "../domain/agent-chat-context-selection"
import type { AgentChatMessageFormatCopy } from "../domain/agent-chat-message-mapping"
import type { AgentChatStrategy } from "../domain/agent-chat-strategy"
import type { AgentChatRuntimeSelection } from "../domain/agent-runtime-options"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
  AgentChatMessage,
} from "../types"
import { sendAgentChatMessage } from "./agent-chat-send-message"

export function useAgentChatSend(input: {
  abortRef: MutableRefObject<AbortController | null>
  attachments: AgentChatAttachmentItem[]
  conversationId: string | null
  fallbackConversationTitle: string
  inputValue: string
  isLoggedIn: boolean
  isStreaming: boolean
  loginRequiredMessage: string
  messageFormatCopy: AgentChatMessageFormatCopy
  runtimeSelection: AgentChatRuntimeSelection
  selectedCampaigns: AgentChatCampaignContextSelection[]
  selectedProducts: AgentChatProductContextSelection[]
  selectedSources: AgentChatSourceContextSelection[]
  sessionIdRef: MutableRefObject<string>
  setConversationId: Dispatch<SetStateAction<string | null>>
  setConversations: Dispatch<SetStateAction<AgentChatConversationItem[]>>
  setError: Dispatch<SetStateAction<string | null>>
  setInput: Dispatch<SetStateAction<string>>
  setIsStreaming: Dispatch<SetStateAction<boolean>>
  setMessages: Dispatch<SetStateAction<AgentChatMessage[]>>
  setRuntimeEvents: Dispatch<SetStateAction<AgentRuntimeEvent[]>>
  strategy: AgentChatStrategy
  tError: string
}) {
  const {
    abortRef,
    attachments,
    conversationId,
    fallbackConversationTitle,
    inputValue,
    isLoggedIn,
    isStreaming,
    loginRequiredMessage,
    messageFormatCopy,
    runtimeSelection,
    selectedCampaigns,
    selectedProducts,
    selectedSources,
    sessionIdRef,
    setConversationId,
    setConversations,
    setError,
    setInput,
    setIsStreaming,
    setMessages,
    setRuntimeEvents,
    strategy,
    tError,
  } = input

  return useCallback(async () => {
    const content = inputValue.trim()
    if (!content || isStreaming) return
    if (!isLoggedIn) {
      setError(loginRequiredMessage)
      return
    }

    const controller = await sendAgentChatMessage({
      content,
      conversationId,
      fallbackConversationTitle,
      messageFormatCopy,
      onController: (controller) => {
        abortRef.current = controller
      },
      attachments,
      runtimeSelection,
      selectedCampaigns,
      selectedProducts,
      selectedSources,
      sessionIdRef,
      setConversationId,
      setConversations,
      setError,
      setInput,
      setIsStreaming,
      setMessages,
      setRuntimeEvents,
      strategy,
      tError,
    })

    if (abortRef.current === controller) {
      abortRef.current = null
    }
  }, [
    abortRef,
    attachments,
    conversationId,
    fallbackConversationTitle,
    inputValue,
    isLoggedIn,
    isStreaming,
    loginRequiredMessage,
    messageFormatCopy,
    runtimeSelection,
    selectedCampaigns,
    selectedProducts,
    selectedSources,
    sessionIdRef,
    setConversationId,
    setConversations,
    setError,
    setInput,
    setIsStreaming,
    setMessages,
    setRuntimeEvents,
    strategy,
    tError,
  ])
}
