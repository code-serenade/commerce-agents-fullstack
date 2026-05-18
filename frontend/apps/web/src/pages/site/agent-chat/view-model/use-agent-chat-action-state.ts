import {
  useCallback,
  useMemo,
  type Dispatch,
  type MutableRefObject,
  type SetStateAction,
} from "react"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import type { AgentChatMessageFormatCopy } from "../domain/agent-chat-message-mapping"
import type { AgentChatRuntimeSelectionRestore } from "../domain/agent-runtime-options"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
  AgentChatMessage,
} from "../types"

export function useAgentChatActionState(input: {
  abortRef: MutableRefObject<AbortController | null>
  clearSelectedCampaigns: () => void
  clearSelectedProducts: () => void
  clearSelectedSources: () => void
  fallbackConversationTitle: string
  messageFormatCopy: AgentChatMessageFormatCopy
  resetRuntimeSelection: () => void
  restoreRuntimeSelection: (selection: AgentChatRuntimeSelectionRestore) => void
  restoreSelectedCampaigns: (campaignIds: string[]) => void
  restoreSelectedProducts: (productIds: string[]) => void
  restoreSelectedSources: (sourceRefs: string[]) => void
  sessionIdRef: MutableRefObject<string>
  setAttachments: Dispatch<SetStateAction<AgentChatAttachmentItem[]>>
  setConversationId: Dispatch<SetStateAction<string | null>>
  setConversations: Dispatch<SetStateAction<AgentChatConversationItem[]>>
  setError: Dispatch<SetStateAction<string | null>>
  setInput: Dispatch<SetStateAction<string>>
  setIsStreaming: Dispatch<SetStateAction<boolean>>
  setMessages: Dispatch<SetStateAction<AgentChatMessage[]>>
  setRuntimeEvents: Dispatch<SetStateAction<AgentRuntimeEvent[]>>
  setStrategyId: Dispatch<SetStateAction<AgentChatStrategyId>>
  tError: string
  welcomeMessage: string
}) {
  const createWelcomeMessage = useCallback(
    (): AgentChatMessage => ({
      content: input.welcomeMessage,
      id: "welcome",
      role: "assistant",
    }),
    [input.welcomeMessage]
  )

  const actionState = useMemo(
    () => ({
      abortRef: input.abortRef,
      clearSelectedCampaigns: input.clearSelectedCampaigns,
      clearSelectedProducts: input.clearSelectedProducts,
      clearSelectedSources: input.clearSelectedSources,
      createWelcomeMessage,
      fallbackConversationTitle: input.fallbackConversationTitle,
      messageFormatCopy: input.messageFormatCopy,
      resetRuntimeSelection: input.resetRuntimeSelection,
      restoreRuntimeSelection: input.restoreRuntimeSelection,
      restoreSelectedCampaigns: input.restoreSelectedCampaigns,
      restoreSelectedProducts: input.restoreSelectedProducts,
      restoreSelectedSources: input.restoreSelectedSources,
      restoreStrategyId: input.setStrategyId,
      sessionIdRef: input.sessionIdRef,
      setAttachments: input.setAttachments,
      setConversationId: input.setConversationId,
      setConversations: input.setConversations,
      setError: input.setError,
      setInput: input.setInput,
      setIsStreaming: input.setIsStreaming,
      setMessages: input.setMessages,
      setRuntimeEvents: input.setRuntimeEvents,
      tError: input.tError,
    }),
    [
      createWelcomeMessage,
      input.abortRef,
      input.clearSelectedCampaigns,
      input.clearSelectedProducts,
      input.clearSelectedSources,
      input.fallbackConversationTitle,
      input.messageFormatCopy,
      input.resetRuntimeSelection,
      input.restoreRuntimeSelection,
      input.restoreSelectedCampaigns,
      input.restoreSelectedProducts,
      input.restoreSelectedSources,
      input.sessionIdRef,
      input.setAttachments,
      input.setConversationId,
      input.setConversations,
      input.setError,
      input.setInput,
      input.setIsStreaming,
      input.setMessages,
      input.setRuntimeEvents,
      input.setStrategyId,
      input.tError,
    ]
  )

  return { actionState, createWelcomeMessage }
}
