import type { ProductResponse } from "@/api"

export interface AgentChatProductContextSelection {
  id: string
  name: string
}

export function mapProductToContextSelection(
  product: ProductResponse
): AgentChatProductContextSelection {
  return {
    id: product.id,
    name: product.name,
  }
}

export function createContextRefs(
  products: AgentChatProductContextSelection[]
): string[] {
  return products.map((product) => `product:${product.id}`)
}

export function toggleProductContextSelection(
  current: AgentChatProductContextSelection[],
  next: AgentChatProductContextSelection
) {
  if (current.some((item) => item.id === next.id)) {
    return current.filter((item) => item.id !== next.id)
  }
  return [...current, next].slice(0, 5)
}
