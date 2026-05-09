import { useCallback, useEffect, useMemo, useState } from "react"
import type { AgentChatProductContextSelection } from "../domain/agent-chat-context-selection"
import {
  loadAgentChatProductContext,
  resetAgentChatProductContext,
  toggleAgentChatProductContext,
} from "./agent-chat-product-context-actions"

export function useAgentChatProductContext(input: {
  enabled: boolean
  isStreaming: boolean
}) {
  const [availableProducts, setAvailableProducts] = useState<AgentChatProductContextSelection[]>([])
  const [selectedProducts, setSelectedProducts] = useState<AgentChatProductContextSelection[]>([])
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
      return
    }

    void loadAgentChatProductContext(state, () => cancelled)

    return () => {
      cancelled = true
    }
  }, [input.enabled, state])

  const toggleProductContext = useCallback(
    (product: AgentChatProductContextSelection) => {
      if (input.isStreaming) return
      toggleAgentChatProductContext(state, product)
    },
    [input.isStreaming, state]
  )

  return {
    availableProducts,
    isLoadingProducts,
    selectedProducts,
    toggleProductContext,
  }
}
