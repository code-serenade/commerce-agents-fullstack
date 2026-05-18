import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import {
  mapAttachmentToSourceContextSelection,
  restoreSourceContextSelection,
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
  const pendingSelectedSourceRefsRef = useRef<string[]>([])
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
    pendingSelectedSourceRefsRef.current = []
    setSelectedSources([])
  }, [isStreaming])

  const restoreSelectedSources = useCallback(
    (sourceRefs: string[]) => {
      if (isStreaming) return
      pendingSelectedSourceRefsRef.current = sourceRefs
      setSelectedSources(
        restoreSourceContextSelection(availableSources, sourceRefs)
      )
    },
    [availableSources, isStreaming]
  )

  useEffect(() => {
    const pendingSelectedSourceRefs = pendingSelectedSourceRefsRef.current
    if (pendingSelectedSourceRefs.length === 0) return
    const restoredSources = restoreSourceContextSelection(
      availableSources,
      pendingSelectedSourceRefs
    )
    if (restoredSources.length === 0) return
    setSelectedSources(restoredSources)
    pendingSelectedSourceRefsRef.current = []
  }, [availableSources])

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
    restoreSelectedSources,
    selectedSources: effectiveSelectedSources,
    toggleSourceContext,
  }
}
