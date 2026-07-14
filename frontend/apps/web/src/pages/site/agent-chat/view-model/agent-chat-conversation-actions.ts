import type { Dispatch, MutableRefObject, SetStateAction } from "react"
import type { ConversationDetailResponse } from "@/api"
import { agentChatData, type AgentRuntimeEvent } from "../agent-chat-data"
import {
  createSessionId,
  formatAgentUiMessage,
  mapConversationAttachments,
  mapConversationItem,
  mapConversationMessages,
  readConversationRuntimeEvents,
  readConversationRuntimeSelection,
  readConversationSelectedCampaignIds,
  readConversationSelectedProductIds,
  readConversationSelectedSourceRefs,
  readConversationStrategyId,
  type AgentChatMessageFormatCopy,
} from "../domain/agent-chat-message-mapping"
import type { AgentChatRuntimeSelectionRestore } from "../domain/agent-runtime-options"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import { resetAgentChatConversationContext } from "./agent-chat-page-actions"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
  AgentChatMessage,
} from "../types"

interface ConversationActionState {
  createWelcomeMessage: () => AgentChatMessage
  fallbackConversationTitle: string
  messageFormatCopy: AgentChatMessageFormatCopy
  sessionIdRef: MutableRefObject<string>
  clearSelectedCampaigns?: () => void
  clearSelectedProducts?: () => void
  clearSelectedSources?: () => void
  resetRuntimeSelection?: () => void
  restoreRuntimeSelection?: (selection: AgentChatRuntimeSelectionRestore) => void
  restoreSelectedCampaigns?: (campaignIds: string[]) => void
  restoreSelectedProducts?: (productIds: string[]) => void
  restoreSelectedSources?: (sourceRefs: string[]) => void
  restoreStrategyId?: (strategyId: AgentChatStrategyId) => void
  setConversationId: Dispatch<SetStateAction<string | null>>
  setConversations: Dispatch<SetStateAction<AgentChatConversationItem[]>>
  setError: Dispatch<SetStateAction<string | null>>
  setInput: Dispatch<SetStateAction<string>>
  setMessages: Dispatch<SetStateAction<AgentChatMessage[]>>
  setRuntimeEvents: Dispatch<SetStateAction<AgentRuntimeEvent[]>>
  setAttachments?: Dispatch<SetStateAction<AgentChatAttachmentItem[]>>
  tError: string
}

export async function loadLatestAgentConversation(
  state: ConversationActionState,
  isCancelled: () => boolean
) {
  try {
    const conversations = await loadAgentConversationList(state, isCancelled)
    if (isCancelled()) return
    const latest = conversations[0]
    if (!latest) {
      resetActiveConversation(state)
      return
    }

    const detail = await agentChatData.getConversation(latest.id)
    if (isCancelled()) return
    state.setConversationId(detail.conversation.id)
    state.sessionIdRef.current = detail.conversation.id
    state.setMessages(
      mapConversationMessages(
        detail,
        state.createWelcomeMessage(),
        state.messageFormatCopy
      )
    )
    state.setRuntimeEvents(readConversationRuntimeEvents(detail))
    restoreConversationContext(detail, state)
  } catch (err) {
    if (isCancelled()) return
    state.setError(formatActionError(err, state))
  }
}

export async function loadAgentConversationList(
  state: ConversationActionState,
  isCancelled: () => boolean
) {
  const conversations = await agentChatData.listConversations()
  if (isCancelled()) return []

  state.setConversations(
    conversations.map((conversation) =>
      mapConversationItem(conversation, state.fallbackConversationTitle)
    )
  )

  return conversations
}

export async function loadAgentConversationById(
  targetConversationId: string,
  state: ConversationActionState,
  isCancelled: () => boolean
) {
  try {
    const detail = await agentChatData.getConversation(targetConversationId)
    if (isCancelled()) return
    restoreActiveConversation(detail, state)
  } catch (err) {
    if (isCancelled()) return
    resetActiveConversation(state)
    state.setError(formatActionError(err, state))
  }
}

export async function selectAgentConversation(
  nextConversationId: string,
  state: ConversationActionState
) {
  state.setError(null)
  try {
    const detail = await agentChatData.getConversation(nextConversationId)
    restoreActiveConversation(detail, state)
  } catch (err) {
    state.setError(formatActionError(err, state))
  }
}

export async function renameAgentConversation(
  targetConversationId: string,
  title: string,
  state: ConversationActionState
) {
  const nextTitle = title.trim()
  if (!nextTitle) return

  state.setError(null)
  try {
    const conversation = await agentChatData.updateConversation(
      targetConversationId,
      { title: nextTitle }
    )
    state.setConversations((current) =>
      current.map((item) =>
        item.id === targetConversationId
          ? mapConversationItem(conversation, state.fallbackConversationTitle)
          : item
      )
    )
  } catch (err) {
    state.setError(formatActionError(err, state))
  }
}

export async function deleteAgentConversation(
  targetConversationId: string,
  activeConversationId: string | null,
  state: ConversationActionState
): Promise<boolean> {
  state.setError(null)
  try {
    await agentChatData.deleteConversation(targetConversationId)
    state.setConversations((current) =>
      current.filter((item) => item.id !== targetConversationId)
    )

    if (targetConversationId === activeConversationId) {
      resetActiveConversation(state)
    }

    return true
  } catch (err) {
    state.setError(formatActionError(err, state))
    return false
  }
}

export function resetActiveConversation(state: ConversationActionState) {
  state.setConversationId(null)
  state.sessionIdRef.current = createSessionId()
  state.setMessages([state.createWelcomeMessage()])
  state.setRuntimeEvents([])
  state.setAttachments?.([])
  state.clearSelectedCampaigns?.()
  state.clearSelectedProducts?.()
  state.clearSelectedSources?.()
  resetAgentChatConversationContext(state)
}

function restoreActiveConversation(
  detail: ConversationDetailResponse,
  state: ConversationActionState
) {
  state.setError(null)
  state.setConversationId(detail.conversation.id)
  state.sessionIdRef.current = detail.conversation.id
  state.setMessages(
    mapConversationMessages(
      detail,
      state.createWelcomeMessage(),
      state.messageFormatCopy
    )
  )
  state.setRuntimeEvents(readConversationRuntimeEvents(detail))
  restoreConversationContext(detail, state)
}

function restoreConversationContext(
  detail: ConversationDetailResponse,
  state: ConversationActionState
) {
  const productIds = readConversationSelectedProductIds(detail)
  const campaignIds = readConversationSelectedCampaignIds(detail)
  const sourceRefs = readConversationSelectedSourceRefs(detail)
  const runtimeSelection = readConversationRuntimeSelection(detail)
  const strategyId = readConversationStrategyId(detail)
  if (productIds.length > 0) {
    state.restoreSelectedProducts?.(productIds)
  } else {
    state.clearSelectedProducts?.()
  }
  if (campaignIds.length > 0) {
    state.restoreSelectedCampaigns?.(campaignIds)
  } else {
    state.clearSelectedCampaigns?.()
  }
  state.setAttachments?.(mapConversationAttachments(detail))
  if (sourceRefs.length > 0) {
    state.restoreSelectedSources?.(sourceRefs)
  } else {
    state.clearSelectedSources?.()
  }
  state.restoreStrategyId?.(strategyId)
  if (runtimeSelection) state.restoreRuntimeSelection?.(runtimeSelection)
}

function formatActionError(err: unknown, state: ConversationActionState) {
  return formatAgentUiMessage(
    err instanceof Error ? err.message : state.tError,
    state.messageFormatCopy
  )
}
