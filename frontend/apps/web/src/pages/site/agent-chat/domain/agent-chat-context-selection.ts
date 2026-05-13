import type { ProductResponse } from "@/api"

export interface AgentChatProductContextSelection {
  brand?: string | null
  category?: string | null
  externalId?: string | null
  id: string
  name: string
  platform?: string | null
  price?: string | null
  status?: string | null
}

export function mapProductToContextSelection(
  product: ProductResponse
): AgentChatProductContextSelection {
  return {
    brand: product.brand,
    category: product.category,
    externalId: product.externalId,
    id: product.id,
    name: product.name,
    platform: product.platform,
    price: product.price,
    status: product.status,
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
