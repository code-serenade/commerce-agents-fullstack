import type { CampaignResponse, ProductResponse } from "@/api"
import type { AgentChatAttachmentItem } from "../types"

export interface AgentChatCampaignContextSelection {
  budget?: number | null
  campaignType?: string | null
  endDate?: string | null
  id: string
  name: string
  productId?: string | null
  startDate?: string | null
  status?: string | null
}

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

export type AgentChatSourceContextKind = "conversation_file"

export interface AgentChatSourceContextSelection {
  id: string
  kind: AgentChatSourceContextKind
  readable: boolean
  ref: string
  status: AgentChatAttachmentItem["status"]
  title: string
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

export function mapCampaignToContextSelection(
  campaign: CampaignResponse
): AgentChatCampaignContextSelection {
  return {
    budget: campaign.budget,
    campaignType: campaign.campaignType,
    endDate: campaign.endDate,
    id: campaign.id,
    name: campaign.name,
    productId: campaign.productId,
    startDate: campaign.startDate,
    status: campaign.status,
  }
}

export function createContextRefs(
  products: AgentChatProductContextSelection[],
  sources: AgentChatSourceContextSelection[] = [],
  campaigns: AgentChatCampaignContextSelection[] = []
): string[] {
  return uniqueContextRefs([
    ...products.map((product) => `product:${product.id}`),
    ...sources.map((source) => source.ref),
    ...campaigns.map((campaign) => `campaign:${campaign.id}`),
  ])
}

export function toggleCampaignContextSelection(
  current: AgentChatCampaignContextSelection[],
  next: AgentChatCampaignContextSelection
) {
  if (current.some((item) => item.id === next.id)) {
    return current.filter((item) => item.id !== next.id)
  }
  return [...current, next].slice(0, 5)
}

export function restoreCampaignContextSelection(
  availableCampaigns: AgentChatCampaignContextSelection[],
  campaignIds: string[]
) {
  const uniqueCampaignIds = uniqueContextRefs(campaignIds)
  return availableCampaigns
    .filter((campaign) => uniqueCampaignIds.includes(campaign.id))
    .slice(0, 5)
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

export function mapAttachmentToSourceContextSelection(
  attachment: AgentChatAttachmentItem
): AgentChatSourceContextSelection | null {
  if (!attachment.fileId || attachment.status !== "uploaded") return null

  return {
    id: attachment.fileId,
    kind: "conversation_file",
    readable: attachment.readable,
    ref: `conversation_file:${attachment.fileId}`,
    status: attachment.status,
    title: attachment.name,
  }
}

export function toggleSourceContextSelection(
  current: AgentChatSourceContextSelection[],
  next: AgentChatSourceContextSelection
) {
  if (current.some((item) => item.ref === next.ref)) {
    return current.filter((item) => item.ref !== next.ref)
  }
  return [...current, next].slice(0, 8)
}

export function restoreSourceContextSelection(
  availableSources: AgentChatSourceContextSelection[],
  sourceRefs: string[]
) {
  const uniqueSourceRefs = uniqueContextRefs(sourceRefs).filter((ref) =>
    ref.startsWith("conversation_file:")
  )
  return availableSources
    .filter((source) => uniqueSourceRefs.includes(source.ref))
    .slice(0, 8)
}

function uniqueContextRefs(refs: string[]) {
  const result: string[] = []
  refs.forEach((ref) => {
    const normalized = ref.trim()
    if (normalized && !result.includes(normalized)) result.push(normalized)
  })
  return result
}
