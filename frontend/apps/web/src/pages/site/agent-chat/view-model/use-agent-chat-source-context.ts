import { useCallback, useMemo, useState } from "react"
import {
  mapAttachmentToSourceContextSelection,
  toggleSourceContextSelection,
  type AgentChatSourceContextSelection,
} from "../domain/agent-chat-context-selection"
import type { AgentChatAttachmentItem } from "../types"

export function useAgentChatSourceContext({
  attachments,
  isStreaming,
}: {
  attachments: AgentChatAttachmentItem[]
  isStreaming: boolean
}) {
  const [selectedSources, setSelectedSources] = useState<
    AgentChatSourceContextSelection[]
  >([])
  const availableSources = useMemo(
    () =>
      attachments
        .map(mapAttachmentToSourceContextSelection)
        .filter((item): item is AgentChatSourceContextSelection =>
          Boolean(item)
        ),
    [attachments]
  )

  const effectiveSelectedSources = useMemo(
    () =>
      selectedSources.filter((selected) =>
        availableSources.some((source) => source.ref === selected.ref)
      ),
    [availableSources, selectedSources]
  )

  const clearSelectedSources = useCallback(() => {
    if (isStreaming) return
    setSelectedSources([])
  }, [isStreaming])

  const toggleSourceContext = useCallback(
    (source: AgentChatSourceContextSelection) => {
      if (isStreaming) return
      setSelectedSources((current) =>
        toggleSourceContextSelection(current, source)
      )
    },
    [isStreaming]
  )

  return {
    availableSources,
    clearSelectedSources,
    selectedSources: effectiveSelectedSources,
    toggleSourceContext,
  }
}
