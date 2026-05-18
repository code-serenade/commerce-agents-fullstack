import { useCallback, type Dispatch, type SetStateAction } from "react"
import type { AgentChatMessageFormatCopy } from "../domain/agent-chat-message-mapping"
import type { AgentChatPlaybookSelection } from "../domain/agent-chat-playbook"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import {
  deleteAgentConversation,
  renameAgentConversation,
  selectAgentConversation,
} from "./agent-chat-conversation-actions"
import {
  startNewAgentConversation,
  stopAgentStreaming,
  type AgentChatPageStateActions,
} from "./agent-chat-page-actions"

interface AgentChatCommandState extends AgentChatPageStateActions {
  fallbackConversationTitle: string
  messageFormatCopy: AgentChatMessageFormatCopy
  tError: string
}

export function useAgentChatPageCommands(input: {
  actionState: AgentChatCommandState
  conversationId: string | null
  isLoggedIn: boolean
  isStreaming: boolean
  setInput: Dispatch<SetStateAction<string>>
  setStrategyId: Dispatch<SetStateAction<AgentChatStrategyId>>
}) {
  const {
    actionState,
    conversationId,
    isLoggedIn,
    isStreaming,
    setInput,
    setStrategyId,
  } = input

  const startNewConversation = useCallback(() => {
    startNewAgentConversation(actionState)
  }, [actionState])

  const selectConversation = useCallback(
    async (nextConversationId: string) => {
      if (!isLoggedIn || nextConversationId === conversationId || isStreaming)
        return
      await selectAgentConversation(nextConversationId, actionState)
    },
    [actionState, conversationId, isLoggedIn, isStreaming]
  )

  const renameConversation = useCallback(
    async (targetConversationId: string, title: string) => {
      if (!isLoggedIn || isStreaming) return
      await renameAgentConversation(targetConversationId, title, actionState)
    },
    [actionState, isLoggedIn, isStreaming]
  )

  const deleteConversation = useCallback(
    async (targetConversationId: string) => {
      if (!isLoggedIn || isStreaming) return
      await deleteAgentConversation(
        targetConversationId,
        conversationId,
        actionState
      )
    },
    [actionState, conversationId, isLoggedIn, isStreaming]
  )

  const stopStreaming = useCallback(() => {
    stopAgentStreaming(actionState)
  }, [actionState])

  const selectPlaybook = useCallback(
    (playbook: AgentChatPlaybookSelection) => {
      if (isStreaming) return
      setInput(playbook.prompt)
      setStrategyId(playbook.recommendedStrategy)
    },
    [isStreaming, setInput, setStrategyId]
  )

  return {
    deleteConversation,
    renameConversation,
    selectConversation,
    selectPlaybook,
    startNewConversation,
    stopStreaming,
  }
}
