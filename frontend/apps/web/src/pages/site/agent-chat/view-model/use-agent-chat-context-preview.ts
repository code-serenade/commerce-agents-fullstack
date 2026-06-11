import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import { agentChatData } from "../agent-chat-data"
import type { AgentContextPreviewResponse } from "../agent-chat-data"
import type {
  AgentChatCampaignContextSelection,
  AgentChatProductContextSelection,
  AgentChatSourceContextSelection,
} from "../domain/agent-chat-context-selection"
import { createContextRefs } from "../domain/agent-chat-context-selection"

export interface AgentChatContextPreviewState {
  canPreviewContext: boolean
  clearContextPreview: () => void
  isPreviewingContext: boolean
  previewContext: () => Promise<void>
  previewContextError: string | null
  previewContextResult: AgentContextPreviewResponse | null
}

export function useAgentChatContextPreview({
  conversationId,
  inputValue,
  isLoggedIn,
  isStreaming,
  selectedCampaigns,
  selectedProducts,
  selectedSources,
  tError,
}: {
  conversationId: string | null
  inputValue: string
  isLoggedIn: boolean
  isStreaming: boolean
  selectedCampaigns: AgentChatCampaignContextSelection[]
  selectedProducts: AgentChatProductContextSelection[]
  selectedSources: AgentChatSourceContextSelection[]
  tError: string
}): AgentChatContextPreviewState {
  const [previewContextResult, setPreviewContextResult] =
    useState<AgentContextPreviewResponse | null>(null)
  const [isPreviewingContext, setIsPreviewingContext] = useState(false)
  const [previewContextError, setPreviewContextError] = useState<string | null>(
    null
  )
  const requestVersionRef = useRef(0)
  const trimmedInput = inputValue.trim()
  const productId = selectedProducts[0]?.id
  const contextRefs = useMemo(
    () =>
      createContextRefs(selectedProducts, selectedSources, selectedCampaigns),
    [selectedCampaigns, selectedProducts, selectedSources]
  )
  const contextFingerprint = contextRefs.join("\n")
  const canPreviewContext =
    isLoggedIn && !isStreaming && trimmedInput.length > 0

  const clearContextPreview = useCallback(() => {
    requestVersionRef.current += 1
    setPreviewContextResult(null)
    setPreviewContextError(null)
    setIsPreviewingContext(false)
  }, [])

  useEffect(() => {
    clearContextPreview()
  }, [
    clearContextPreview,
    contextFingerprint,
    conversationId,
    isLoggedIn,
    isStreaming,
    productId,
    trimmedInput,
  ])

  const previewContext = useCallback(async () => {
    if (!canPreviewContext) return

    const requestVersion = requestVersionRef.current + 1
    requestVersionRef.current = requestVersion
    setIsPreviewingContext(true)
    setPreviewContextError(null)

    try {
      const preview = await agentChatData.previewAgentContext({
        contextRefs: contextRefs.length ? contextRefs : undefined,
        conversationId: conversationId ?? undefined,
        input: trimmedInput,
        productId,
      })
      if (requestVersionRef.current !== requestVersion) return
      setPreviewContextResult(preview)
    } catch (err) {
      if (requestVersionRef.current !== requestVersion) return
      setPreviewContextError(
        err instanceof Error && err.message ? err.message : tError
      )
      setPreviewContextResult(null)
    } finally {
      if (requestVersionRef.current === requestVersion) {
        setIsPreviewingContext(false)
      }
    }
  }, [
    canPreviewContext,
    contextRefs,
    conversationId,
    productId,
    tError,
    trimmedInput,
  ])

  return {
    canPreviewContext,
    clearContextPreview,
    isPreviewingContext,
    previewContext,
    previewContextError,
    previewContextResult,
  }
}
