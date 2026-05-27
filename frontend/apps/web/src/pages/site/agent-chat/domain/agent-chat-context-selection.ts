import type {
  CampaignResponse,
  ConversationFileResponse,
  ProductResponse,
} from "@/api"
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
  bucket?: string
  id: string
  kind: AgentChatSourceContextKind
  objectKey?: string
  readable: boolean
  ref: string
  status: AgentChatAttachmentItem["status"]
  title: string
  visionReady?: boolean
}

export type AgentChatSourceLibraryStatus =
  | "readable"
  | "pending"
  | "failed"
  | "unsupported"

export interface AgentChatSourceLibraryItem {
  attached: boolean
  bucket: string
  createdAt: string
  fileSize: number | null
  id: string
  mimeType: string | null
  objectKey: string
  readable: boolean
  status: AgentChatSourceLibraryStatus
  title: string
  visionReady: boolean
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
  if (
    !attachment.fileId ||
    !["uploaded", "pending", "unsupported"].includes(attachment.status)
  ) {
    return null
  }

  return {
    bucket: attachment.bucket,
    id: attachment.fileId,
    kind: "conversation_file",
    objectKey: attachment.objectKey,
    readable: attachment.readable,
    ref: `conversation_file:${attachment.fileId}`,
    status: attachment.status,
    title: attachment.name,
    visionReady: Boolean(attachment.imageDataUrl),
  }
}

export function mapConversationFileToSourceLibraryItem(
  file: ConversationFileResponse,
  attachedObjectKeys: Set<string> = new Set()
): AgentChatSourceLibraryItem {
  const status = conversationFileReadableState(file)
  const visionReady = isVisionConversationFile(file)

  return {
    attached: attachedObjectKeys.has(conversationFileObjectKey(file)),
    bucket: file.bucket,
    createdAt: file.createdAt,
    fileSize: file.fileSize,
    id: file.id,
    mimeType: file.mimeType,
    objectKey: file.objectKey,
    readable: status === "readable" || visionReady,
    status,
    title: file.fileName || file.objectKey.split("/").at(-1) || file.id,
    visionReady,
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

function conversationFileObjectKey(file: ConversationFileResponse) {
  return `${file.bucket}:${file.objectKey}`
}

function conversationFileReadableState(
  file: ConversationFileResponse
): AgentChatSourceLibraryStatus {
  const extraction = metadataRecord(file.metadata, "extraction")
  const status =
    metadataString(file.metadata, "extractionStatus") ||
    (extraction ? metadataString(extraction, "status") : undefined)
  if (status === "extracted" || status === "ready") return "readable"
  if (status === "failed") return "failed"
  if (status === "unsupported") return "unsupported"
  return "pending"
}

function isVisionConversationFile(file: ConversationFileResponse) {
  const mimeType = file.mimeType?.toLowerCase() ?? ""
  return (
    ["image/png", "image/jpeg", "image/webp"].includes(mimeType) ||
    /\.(png|jpe?g|webp)$/i.test(file.fileName ?? file.objectKey)
  )
}

function metadataString(
  metadata: Record<string, unknown>,
  key: string
): string | undefined {
  const value = metadata[key]
  return typeof value === "string" && value.trim() ? value : undefined
}

function metadataRecord(
  metadata: Record<string, unknown>,
  key: string
): Record<string, unknown> | undefined {
  const value = metadata[key]
  return typeof value === "object" && value !== null
    ? (value as Record<string, unknown>)
    : undefined
}
