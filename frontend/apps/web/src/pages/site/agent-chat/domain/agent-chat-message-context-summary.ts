import type { AgentChatMessageContextSummary } from "../types"
import { normalizeAgentChatStrategyId } from "./agent-chat-strategy"

export interface MessageContextSummaryProductInput {
  brand?: string | null
  category?: string | null
  id: string
  name?: string | null
  platform?: string | null
  status?: string | null
}

export interface MessageContextSummarySourceInput {
  ref: string
  title?: string | null
}

export function createMessageContextSummary(input: {
  collaborationMode?: string
  contextRefs: string[]
  learningLevel?: string
  modelName?: string
  primaryRole?: string
  productIds: string[]
  products?: MessageContextSummaryProductInput[]
  selectedRoles?: string[]
  sources?: MessageContextSummarySourceInput[]
  strategyId?: string
}): AgentChatMessageContextSummary | undefined {
  const productIds = uniqueStrings([
    ...input.productIds,
    ...input.contextRefs
      .map((item) => item.match(/^product:(.+)$/)?.[1])
      .filter((item): item is string => Boolean(item)),
  ])
  const sourceRefs = uniqueStrings(
    input.contextRefs.filter((ref) => ref.startsWith("conversation_file:"))
  )
  const selectedRoles = uniqueStrings(input.selectedRoles ?? [])
  const products = productIds.map((id) => {
    const product = input.products?.find((item) => item.id === id)
    return {
      id,
      ...(product?.name ? { label: product.name } : {}),
      ...getProductSecondary(product),
    }
  })
  const sources = sourceRefs.map((ref) => {
    const source = input.sources?.find((item) => item.ref === ref)
    return {
      id: ref,
      ...(source?.title ? { label: source.title } : {}),
    }
  })
  const strategyId =
    normalizeAgentChatStrategyId(input.strategyId) ?? undefined
  const summary: AgentChatMessageContextSummary = {
    productIds,
    products,
    selectedRoles,
    sourceRefs,
    sources,
  }

  if (strategyId) summary.strategyId = strategyId
  if (input.collaborationMode) summary.collaborationMode = input.collaborationMode
  if (input.learningLevel) summary.learningLevel = input.learningLevel
  if (input.modelName) summary.modelName = input.modelName
  if (input.primaryRole) summary.primaryRole = input.primaryRole

  return summaryHasContent(summary) ? summary : undefined
}

export function readMessageContextSummary(
  contentJson: unknown
): AgentChatMessageContextSummary | undefined {
  if (!isRecord(contentJson)) return undefined

  const context = contentJson.context
  const hasContext = isRecord(context)
  const strategy = readStrategyRecord(contentJson)
  if (!hasContext && !strategy) return undefined

  return createMessageContextSummary({
    collaborationMode: readString(strategy?.collaborationMode) ?? undefined,
    contextRefs: hasContext ? readStringArray(context.contextRefs) : [],
    learningLevel: readString(strategy?.learningLevel) ?? undefined,
    modelName: readString(strategy?.modelName) ?? undefined,
    primaryRole: readString(strategy?.primaryRole) ?? undefined,
    productIds: hasContext ? readStringArray(context.productIds) : [],
    selectedRoles: readStringArray(strategy?.selectedRoles),
    strategyId: readString(strategy?.strategyId) ?? undefined,
  })
}

function readStrategyRecord(contentJson: unknown) {
  if (!isRecord(contentJson)) return null
  const strategy = contentJson.strategy
  return isRecord(strategy) ? strategy : null
}

function readStringArray(value: unknown) {
  if (!Array.isArray(value)) return []
  return value.filter((item): item is string => typeof item === "string")
}

function readString(value: unknown) {
  return typeof value === "string" && value.trim() ? value : null
}

function uniqueStrings(values: string[]) {
  const result: string[] = []
  values.forEach((value) => {
    const normalized = value.trim()
    if (normalized && !result.includes(normalized)) result.push(normalized)
  })
  return result
}

function getProductSecondary(
  product: MessageContextSummaryProductInput | undefined
) {
  if (!product) return {}
  const secondary = uniqueStrings([
    product.brand ?? "",
    product.platform ?? "",
    product.category ?? "",
    product.status ?? "",
  ]).join(" / ")

  return secondary ? { secondary } : {}
}

function summaryHasContent(summary: AgentChatMessageContextSummary) {
  return Boolean(
    summary.productIds.length ||
      summary.sourceRefs.length ||
      summary.strategyId ||
      summary.collaborationMode ||
      summary.learningLevel ||
      summary.modelName ||
      summary.primaryRole ||
      summary.selectedRoles.length
  )
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null
}
