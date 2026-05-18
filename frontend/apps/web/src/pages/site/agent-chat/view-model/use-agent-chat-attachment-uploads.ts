import { useCallback, type MutableRefObject } from "react"
import type { Dispatch, SetStateAction } from "react"
import {
  formatAgentUiMessage,
  type AgentChatMessageFormatCopy,
} from "../domain/agent-chat-message-mapping"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
} from "../types"
import { uploadAgentChatAttachments } from "./agent-chat-attachments"

export function useAgentChatAttachmentUploads(input: {
  conversationId: string | null
  fallbackConversationTitle: string
  isLoggedIn: boolean
  isStreaming: boolean
  messageFormatCopy: AgentChatMessageFormatCopy
  sessionIdRef: MutableRefObject<string>
  setAttachments: Dispatch<SetStateAction<AgentChatAttachmentItem[]>>
  setConversationId: Dispatch<SetStateAction<string | null>>
  setConversations: Dispatch<SetStateAction<AgentChatConversationItem[]>>
  setError: Dispatch<SetStateAction<string | null>>
  tError: string
}) {
  const {
    conversationId,
    fallbackConversationTitle,
    isLoggedIn,
    isStreaming,
    messageFormatCopy,
    sessionIdRef,
    setAttachments,
    setConversationId,
    setConversations,
    setError,
    tError,
  } = input

  return useCallback(
    async (files: File[]) => {
      if (!isLoggedIn || isStreaming || files.length === 0) return
      setError(null)
      try {
        const activeConversationId = await uploadAgentChatAttachments({
          conversationId,
          fallbackConversationTitle,
          files,
          setAttachments,
          setConversationId,
          setConversations,
        })
        if (activeConversationId) {
          sessionIdRef.current = activeConversationId
        }
      } catch (err) {
        setError(
          formatAgentUiMessage(
            err instanceof Error ? err.message : tError,
            messageFormatCopy
          )
        )
      }
    },
    [
      conversationId,
      fallbackConversationTitle,
      isLoggedIn,
      isStreaming,
      messageFormatCopy,
      sessionIdRef,
      setAttachments,
      setConversationId,
      setConversations,
      setError,
      tError,
    ]
  )
}
