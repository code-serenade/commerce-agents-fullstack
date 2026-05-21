import type { Dispatch, SetStateAction } from "react"
import { agentChatData } from "../agent-chat-data"
import {
  mapProductToContextSelection,
  toggleProductContextSelection,
  type AgentChatProductContextSelection,
} from "../domain/agent-chat-context-selection"

export function restoreAgentChatProductContext(
  state: {
    setSelectedProducts: Dispatch<SetStateAction<AgentChatProductContextSelection[]>>
  },
  availableProducts: AgentChatProductContextSelection[],
  productIds: string[]
) {
  const uniqueProductIds = Array.from(new Set(productIds))
  state.setSelectedProducts(
    availableProducts
      .filter((product) => uniqueProductIds.includes(product.id))
      .slice(0, 5)
  )
}

export async function fetchAgentChatProductContext() {
  const productGroups = await Promise.all([
    agentChatData.listProducts({ status: "active" }),
    agentChatData.listProducts({ status: "listing" }),
    agentChatData.listProducts({ status: "draft" }),
  ])
  return productGroups
    .flat()
    .map(mapProductToContextSelection)
    .filter(
      (product, index, items) =>
        items.findIndex((item) => item.id === product.id) === index
    )
}

export function toggleAgentChatProductContext(
  state: {
    setSelectedProducts: Dispatch<SetStateAction<AgentChatProductContextSelection[]>>
  },
  product: AgentChatProductContextSelection
) {
  state.setSelectedProducts((current) =>
    toggleProductContextSelection(current, product)
  )
}
