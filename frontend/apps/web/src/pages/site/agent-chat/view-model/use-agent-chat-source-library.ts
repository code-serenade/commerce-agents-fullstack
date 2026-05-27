import { useMemo } from "react"
import { useQuery } from "@tanstack/react-query"
import { agentChatData } from "../agent-chat-data"
import {
  mapConversationFileToSourceLibraryItem,
  type AgentChatSourceContextSelection,
  type AgentChatSourceLibraryItem,
} from "../domain/agent-chat-context-selection"

export function useAgentChatSourceLibrary({
  availableSources,
  enabled,
}: {
  availableSources: AgentChatSourceContextSelection[]
  enabled: boolean
}) {
  const filesQuery = useQuery({
    queryKey: ["agent-chat", "source-library", 24],
    queryFn: () => agentChatData.listUserFiles({ limit: 24 }),
    enabled,
  })
  const attachedObjectKeys = useMemo(
    () =>
      new Set(
        availableSources
          .filter((source) => source.bucket && source.objectKey)
          .map((source) => `${source.bucket}:${source.objectKey}`)
      ),
    [availableSources]
  )
  const sourceLibrary = useMemo<AgentChatSourceLibraryItem[]>(
    () =>
      enabled
        ? (filesQuery.data ?? [])
            .map((file) =>
              mapConversationFileToSourceLibraryItem(file, attachedObjectKeys)
            )
            .slice(0, 8)
        : [],
    [attachedObjectKeys, enabled, filesQuery.data]
  )
  const error =
    enabled && filesQuery.error
      ? filesQuery.error instanceof Error
        ? filesQuery.error.message
        : "Failed to load source library."
      : null

  return {
    isLoadingSourceLibrary: enabled ? filesQuery.isLoading : false,
    sourceLibrary,
    sourceLibraryError: error,
  }
}
