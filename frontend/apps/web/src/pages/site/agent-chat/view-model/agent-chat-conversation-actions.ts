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
  readConversationSelectedProductIds,
  type AgentChatMessageFormatCopy,
} from "../domain/agent-chat-message-mapping"
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
  clearSelectedProducts?: () => void
  restoreSelectedProducts?: (productIds: string[]) => void
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
    const conversations = await agentChatData.listConversations()
    if (isCancelled()) return

    state.setConversations(
      conversations.map((conversation) =>
        mapConversationItem(conversation, state.fallbackConversationTitle)
      )
    )
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

export async function selectAgentConversation(
  nextConversationId: string,
  state: ConversationActionState
) {
  state.setError(null)
  try {
    const detail = await agentChatData.getConversation(nextConversationId)
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
) {
  state.setError(null)
  try {
    await agentChatData.deleteConversation(targetConversationId)
    state.setConversations((current) =>
      current.filter((item) => item.id !== targetConversationId)
    )

    if (targetConversationId === activeConversationId) {
      resetActiveConversation(state)
      state.setRuntimeEvents([])
      state.setInput("")
      state.clearSelectedProducts?.()
    }
  } catch (err) {
    state.setError(formatActionError(err, state))
  }
}

function resetActiveConversation(state: ConversationActionState) {
  state.setConversationId(null)
  state.sessionIdRef.current = createSessionId()
  state.setMessages([state.createWelcomeMessage()])
  state.setRuntimeEvents([])
  state.setAttachments?.([])
  state.clearSelectedProducts?.()
}

function restoreConversationContext(
  detail: ConversationDetailResponse,
  state: ConversationActionState
) {
  const productIds = readConversationSelectedProductIds(detail)
  if (productIds.length > 0) {
    state.restoreSelectedProducts?.(productIds)
  } else {
    state.clearSelectedProducts?.()
  }
  state.setAttachments?.(mapConversationAttachments(detail))
}

function formatActionError(err: unknown, state: ConversationActionState) {
  return formatAgentUiMessage(
    err instanceof Error ? err.message : state.tError,
    state.messageFormatCopy
  )
}
