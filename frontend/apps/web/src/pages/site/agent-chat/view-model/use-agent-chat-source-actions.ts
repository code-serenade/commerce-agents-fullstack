import { useCallback } from "react"
import type { Dispatch, MutableRefObject, SetStateAction } from "react"
import { useQueryClient } from "@tanstack/react-query"
import { AGENT_ID, agentChatData } from "../agent-chat-data"
import type { AgentChatSourceContextSelection } from "../domain/agent-chat-context-selection"
import { mapConversationItem } from "../domain/agent-chat-message-mapping"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
} from "../types"
import {
  deleteAgentChatAttachments,
  mergeConversationFileAttachment,
} from "./agent-chat-attachments"

export function useAgentChatSourceActions({
  conversationId,
  fallbackConversationTitle,
  isLoggedIn,
  isStreaming,
  selectedSources,
  sessionIdRef,
  setAttachments,
  setConversationId,
  setConversations,
  setError,
  restoreSelectedSources,
}: {
  conversationId: string | null
  fallbackConversationTitle: string
  isLoggedIn: boolean
  isStreaming: boolean
  selectedSources: AgentChatSourceContextSelection[]
  sessionIdRef: MutableRefObject<string>
  setAttachments: Dispatch<SetStateAction<AgentChatAttachmentItem[]>>
  setConversationId: Dispatch<SetStateAction<string | null>>
  setConversations: Dispatch<SetStateAction<AgentChatConversationItem[]>>
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
          const detail = await agentChatData.createConversation({
            title: fallbackConversationTitle,
            agentId: AGENT_ID,
          })
          activeConversationId = detail.conversation.id
          sessionIdRef.current = activeConversationId
          setConversationId(activeConversationId)
          setConversations((current) => [
            mapConversationItem(detail.conversation, fallbackConversationTitle),
            ...current.filter((item) => item.id !== detail.conversation.id),
          ])
        }
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
      fallbackConversationTitle,
      invalidateFileQueries,
      isLoggedIn,
      isStreaming,
      restoreSelectedSources,
      selectedSources,
      setAttachments,
      setConversationId,
      setConversations,
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
