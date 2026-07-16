import { useCallback } from "react"
import type { Dispatch, MutableRefObject, SetStateAction } from "react"
import { useQueryClient } from "@tanstack/react-query"
import { agentChatData } from "../agent-chat-data"
import type { AgentChatSourceContextSelection } from "../domain/agent-chat-context-selection"
import type { AgentChatAttachmentItem } from "../types"
import {
  deleteAgentChatAttachments,
  mergeConversationFileAttachment,
} from "./agent-chat-attachments"
import type { EnsureAgentChatConversation } from "./use-agent-chat-conversation-identity"

export function useAgentChatSourceActions({
  conversationId,
  ensureConversation,
  fallbackConversationTitle,
  isLoggedIn,
  isStreaming,
  selectedSources,
  sessionIdRef,
  setAttachments,
  setError,
  restoreSelectedSources,
}: {
  conversationId: string | null
  ensureConversation: EnsureAgentChatConversation
  fallbackConversationTitle: string
  isLoggedIn: boolean
  isStreaming: boolean
  selectedSources: AgentChatSourceContextSelection[]
  sessionIdRef: MutableRefObject<string>
  setAttachments: Dispatch<SetStateAction<AgentChatAttachmentItem[]>>
  setError: Dispatch<SetStateAction<string | null>>
  restoreSelectedSources: (sourceRefs: string[]) => void
}) {
  const queryClient = useQueryClient()

  const invalidateFileQueries = useCallback(async () => {
    await Promise.all([
      queryClient.invalidateQueries({
        queryKey: ["agent-chat", "source-library"],
      }),
      queryClient.invalidateQueries({ queryKey: ["commerce", "files"] }),
    ])
  }, [queryClient])

  const removeSourcesByObjects = useCallback(
    (objects: { bucket: string; objectKey: string }[]) => {
      if (!objects.length) return
      const objectKeys = new Set(
        objects.map((object) => `${object.bucket}:${object.objectKey}`)
      )
      setAttachments((current) =>
        current.filter((item) => {
          if (!item.bucket || !item.objectKey) return true
          return !objectKeys.has(`${item.bucket}:${item.objectKey}`)
        })
      )
    },
    [setAttachments]
  )

  const deleteSelectedSources = useCallback(async () => {
    if (!isLoggedIn || isStreaming || selectedSources.length === 0) return
    setError(null)
    try {
      await deleteAgentChatAttachments({
        fileIds: selectedSources.map((source) => source.id),
        setAttachments,
      })
      await invalidateFileQueries()
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete files.")
    }
  }, [
    invalidateFileQueries,
    isLoggedIn,
    isStreaming,
    selectedSources,
    setAttachments,
    setError,
  ])

  const attachExistingSource = useCallback(
    async (fileId: string) => {
      if (!isLoggedIn || isStreaming) return
      setError(null)
      try {
        let activeConversationId = conversationId
        if (!activeConversationId) {
          activeConversationId = await ensureConversation({
            title: fallbackConversationTitle,
          })
        }
        if (!activeConversationId) return
        sessionIdRef.current = activeConversationId
        const file = await agentChatData.attachConversationFile(fileId, {
          conversationId: activeConversationId,
        })
        setAttachments((current) =>
          mergeConversationFileAttachment(current, file)
        )
        restoreSelectedSources([
          ...selectedSources.map((source) => source.ref),
          `conversation_file:${file.id}`,
        ])
        await invalidateFileQueries()
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to attach file.")
      }
    },
    [
      conversationId,
      ensureConversation,
      fallbackConversationTitle,
      invalidateFileQueries,
      isLoggedIn,
      isStreaming,
      restoreSelectedSources,
      selectedSources,
      setAttachments,
      setError,
      sessionIdRef,
    ]
  )

  return {
    attachExistingSource,
    deleteSelectedSources,
    removeSourcesByObjects,
  }
}
