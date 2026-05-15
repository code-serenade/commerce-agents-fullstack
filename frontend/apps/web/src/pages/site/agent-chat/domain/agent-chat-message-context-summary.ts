import type { AgentChatMessageContextSummary } from "../types"
import { isAgentChatStrategyId } from "./agent-chat-strategy"

export function createMessageContextSummary(input: {
  collaborationMode?: string
  contextRefs: string[]
  learningLevel?: string
  modelName?: string
  primaryRole?: string
  productIds: string[]
  selectedRoles?: string[]
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
  const strategyId =
    input.strategyId && isAgentChatStrategyId(input.strategyId)
      ? input.strategyId
      : undefined
  const summary: AgentChatMessageContextSummary = {
    productIds,
    selectedRoles,
    sourceRefs,
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
