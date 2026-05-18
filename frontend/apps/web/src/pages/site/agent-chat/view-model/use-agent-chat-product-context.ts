import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import type { AgentChatProductContextSelection } from "../domain/agent-chat-context-selection"
import {
  loadAgentChatProductContext,
  resetAgentChatProductContext,
  restoreAgentChatProductContext,
  toggleAgentChatProductContext,
} from "./agent-chat-product-context-actions"

export function useAgentChatProductContext(input: {
  enabled: boolean
  isStreaming: boolean
}) {
  const [availableProducts, setAvailableProducts] = useState<AgentChatProductContextSelection[]>([])
  const [selectedProducts, setSelectedProducts] = useState<AgentChatProductContextSelection[]>([])
  const pendingSelectedProductIdsRef = useRef<string[]>([])
  const [isLoadingProducts, setIsLoadingProducts] = useState(false)
  const state = useMemo(
    () => ({
      setAvailableProducts,
      setIsLoadingProducts,
      setSelectedProducts,
    }),
    []
  )

  useEffect(() => {
    let cancelled = false

    if (!input.enabled) {
      resetAgentChatProductContext(state)
      pendingSelectedProductIdsRef.current = []
      return
    }

    void loadAgentChatProductContext(state, () => cancelled)

    return () => {
      cancelled = true
    }
  }, [input.enabled, state])

  useEffect(() => {
    const pendingSelectedProductIds = pendingSelectedProductIdsRef.current
    if (isLoadingProducts || pendingSelectedProductIds.length === 0) return
    restoreAgentChatProductContext(
      state,
      availableProducts,
      pendingSelectedProductIds
    )
    pendingSelectedProductIdsRef.current = []
  }, [availableProducts, isLoadingProducts, state])

  const toggleProductContext = useCallback(
    (product: AgentChatProductContextSelection) => {
      if (input.isStreaming) return
      toggleAgentChatProductContext(state, product)
    },
    [input.isStreaming, state]
  )
  const clearSelectedProducts = useCallback(() => {
    if (input.isStreaming) return
    pendingSelectedProductIdsRef.current = []
    setSelectedProducts([])
  }, [input.isStreaming])
  const restoreSelectedProducts = useCallback(
    (productIds: string[]) => {
      if (input.isStreaming) return
      const uniqueProductIds = Array.from(new Set(productIds))
      pendingSelectedProductIdsRef.current = uniqueProductIds
      restoreAgentChatProductContext(state, availableProducts, uniqueProductIds)
    },
    [availableProducts, input.isStreaming, state]
  )

  return {
    availableProducts,
    clearSelectedProducts,
    isLoadingProducts,
    restoreSelectedProducts,
    selectedProducts,
    toggleProductContext,
  }
}
