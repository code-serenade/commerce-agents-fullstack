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
import type { AgentChatSourceContextSelection } from "../domain/agent-chat-context-selection"
import { uploadAgentChatAttachments } from "./agent-chat-attachments"

export function useAgentChatAttachmentUploads(input: {
  conversationId: string | null
  fallbackConversationTitle: string
  isLoggedIn: boolean
  isStreaming: boolean
  messageFormatCopy: AgentChatMessageFormatCopy
  onConversationCreated?: (conversationId: string) => void
  sessionIdRef: MutableRefObject<string>
  restoreSelectedSources: (sourceRefs: string[]) => void
  selectedSources: AgentChatSourceContextSelection[]
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
    onConversationCreated,
    restoreSelectedSources,
    selectedSources,
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
        const result = await uploadAgentChatAttachments({
          conversationId,
          fallbackConversationTitle,
          files,
          setAttachments,
          setConversationId,
          setConversations,
        })
        if (result.conversationId) {
          sessionIdRef.current = result.conversationId
          if (!conversationId) {
            onConversationCreated?.(result.conversationId)
          }
        }
        if (result.fileIds.length) {
          const sourceRefs = [
            ...selectedSources.map((source) => source.ref),
            ...result.fileIds.map((fileId) => `conversation_file:${fileId}`),
          ]
          restoreSelectedSources(Array.from(new Set(sourceRefs)))
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
      onConversationCreated,
      restoreSelectedSources,
      selectedSources,
      sessionIdRef,
      setAttachments,
      setConversationId,
      setConversations,
      setError,
      tError,
    ]
  )
}
