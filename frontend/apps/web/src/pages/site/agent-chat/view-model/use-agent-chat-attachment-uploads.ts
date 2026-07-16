import { useCallback, type MutableRefObject } from "react"
import type { Dispatch, SetStateAction } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "@workspace/ui-components"
import {
  formatAgentUiMessage,
  type AgentChatMessageFormatCopy,
} from "../domain/agent-chat-message-mapping"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
} from "../types"
import type { AgentChatSourceContextSelection } from "../domain/agent-chat-context-selection"
import {
  MAX_AI_READABLE_UPLOAD_BYTES,
  validateAiReadableUploads,
  type UploadValidationError,
} from "../domain/agent-chat-upload-validation"
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
  const { t } = useTranslation()
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
      const validation = validateAiReadableUploads(files)
      validation.errors.forEach((error) =>
        toast.error(uploadValidationMessage(error, t))
      )
      if (validation.acceptedFiles.length === 0) return
      try {
        const result = await uploadAgentChatAttachments({
          conversationId,
          fallbackConversationTitle,
          files: validation.acceptedFiles,
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
      t,
    ]
  )
}

function uploadValidationMessage(
  error: UploadValidationError,
  t: (key: string, options?: Record<string, unknown>) => string
) {
  const options = {
    fileName: error.file.name,
    maxSize: formatUploadSize(MAX_AI_READABLE_UPLOAD_BYTES),
  }
  return t(`agentChat.composer.uploadValidation.${error.code}`, options)
}

function formatUploadSize(bytes: number) {
  return `${bytes / (1024 * 1024)} MB`
}
