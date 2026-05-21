import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { productsQueryKey } from "../../commerce/products/use-product-queries"
import type { AgentChatProductContextSelection } from "../domain/agent-chat-context-selection"
import {
  fetchAgentChatProductContext,
  restoreAgentChatProductContext,
  toggleAgentChatProductContext,
} from "./agent-chat-product-context-actions"

const agentChatProductContextQueryKey = [
  ...productsQueryKey,
  "context",
  "agent-chat",
]

export function useAgentChatProductContext(input: {
  enabled: boolean
  isStreaming: boolean
}) {
  const [selectedProductState, setSelectedProducts] = useState<AgentChatProductContextSelection[]>([])
  const pendingSelectedProductIdsRef = useRef<string[]>([])
  const productsQuery = useQuery({
    enabled: input.enabled,
    queryFn: fetchAgentChatProductContext,
    queryKey: agentChatProductContextQueryKey,
  })
  const availableProducts = useMemo(
    () => (input.enabled ? productsQuery.data ?? [] : []),
    [input.enabled, productsQuery.data]
  )
  const state = useMemo(
    () => ({
      setSelectedProducts,
    }),
    []
  )
  useEffect(() => {
    if (!input.enabled) pendingSelectedProductIdsRef.current = []
  }, [input.enabled])
  const selectedProducts = useMemo(
    () => {
      if (!input.enabled) return []

      const availableProductMap = new Map(
        availableProducts.map((product) => [product.id, product])
      )
      return selectedProductState
        .map((selected) => availableProductMap.get(selected.id))
        .filter(
          (product): product is AgentChatProductContextSelection =>
            product !== undefined
        )
    },
    [availableProducts, input.enabled, selectedProductState]
  )

  useEffect(() => {
    const pendingSelectedProductIds = pendingSelectedProductIdsRef.current
    if (productsQuery.isLoading || pendingSelectedProductIds.length === 0) return
    restoreAgentChatProductContext(
      state,
      availableProducts,
      pendingSelectedProductIds
    )
    pendingSelectedProductIdsRef.current = []
  }, [availableProducts, productsQuery.isLoading, state])

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
    isLoadingProducts: productsQuery.isLoading,
    restoreSelectedProducts,
    selectedProducts,
    toggleProductContext,
  }
}
