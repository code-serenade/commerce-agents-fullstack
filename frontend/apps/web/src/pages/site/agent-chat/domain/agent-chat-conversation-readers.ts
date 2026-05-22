import type { ConversationDetailResponse } from "@/api"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import type { AgentChatRuntimeSelectionRestore } from "./agent-runtime-options"
import {
  DEFAULT_AGENT_CHAT_STRATEGY_ID,
  normalizeAgentChatStrategyId,
  type AgentChatStrategyId,
} from "./agent-chat-strategy"

export function readConversationRuntimeEvents(
  detail: ConversationDetailResponse
): AgentRuntimeEvent[] {
  for (let index = detail.messages.length - 1; index >= 0; index -= 1) {
    const runtimeEvents = readRuntimeEvents(detail.messages[index]?.contentJson)
    if (runtimeEvents.length > 0) return runtimeEvents
  }
  return []
}

export function readConversationSelectedProductIds(
  detail: ConversationDetailResponse
): string[] {
  return readLatestUserContext(detail, readProductIds)
}

export function readConversationSelectedSourceRefs(
  detail: ConversationDetailResponse
): string[] {
  return readLatestUserContext(detail, readSourceRefs)
}

export function readConversationSelectedCampaignIds(
  detail: ConversationDetailResponse
): string[] {
  return readLatestUserContext(detail, readCampaignIds)
}

export function readConversationRuntimeSelection(
  detail: ConversationDetailResponse
): AgentChatRuntimeSelectionRestore | null {
  for (let index = detail.messages.length - 1; index >= 0; index -= 1) {
    const message = detail.messages[index]
    if (message?.role !== "user") continue
    const selection = readRuntimeSelection(message.contentJson)
    if (selection) return selection
  }
  return null
}

export function readConversationStrategyId(
  detail: ConversationDetailResponse
): AgentChatStrategyId {
  for (let index = detail.messages.length - 1; index >= 0; index -= 1) {
    const message = detail.messages[index]
    if (message?.role !== "user") continue
    const strategyId = readStrategyId(message.contentJson)
    if (strategyId) return strategyId
  }
  return DEFAULT_AGENT_CHAT_STRATEGY_ID
}

export function isAgentRuntimeEvent(value: unknown): value is AgentRuntimeEvent {
  return isRecord(value) && typeof value.type === "string"
}

function readLatestUserContext(
  detail: ConversationDetailResponse,
  reader: (contentJson: unknown) => string[]
) {
  for (let index = detail.messages.length - 1; index >= 0; index -= 1) {
    const message = detail.messages[index]
    if (message?.role !== "user") continue
    const values = reader(message.contentJson)
    if (values.length > 0) return values
  }
  return []
}

function readRuntimeEvents(contentJson: unknown): AgentRuntimeEvent[] {
  if (!isRecord(contentJson)) return []
  const runtimeEvents = contentJson.runtimeEvents
  if (!Array.isArray(runtimeEvents)) return []
  return runtimeEvents.filter(isAgentRuntimeEvent)
}

function readProductIds(contentJson: unknown): string[] {
  return readContextIds(contentJson, "productIds", /^product:(.+)$/)
}

function readCampaignIds(contentJson: unknown): string[] {
  return readContextIds(contentJson, "campaignIds", /^campaign:(.+)$/)
}

function readSourceRefs(contentJson: unknown): string[] {
  const context = readContext(contentJson)
  if (!context) return []
  return readStringArray(context.contextRefs).filter((ref) =>
    ref.startsWith("conversation_file:")
  )
}

function readContextIds(
  contentJson: unknown,
  idKey: "campaignIds" | "productIds",
  refPattern: RegExp
) {
  const context = readContext(contentJson)
  if (!context) return []
  const ids = readStringArray(context[idKey])
  const refIds = readStringArray(context.contextRefs)
    .map((item) => item.match(refPattern)?.[1])
    .filter((item): item is string => Boolean(item))
  return uniqueStrings([...ids, ...refIds])
}

function readRuntimeSelection(
  contentJson: unknown
): AgentChatRuntimeSelectionRestore | null {
  const strategy = readStrategyRecord(contentJson)
  if (!strategy) return null
  const selection: AgentChatRuntimeSelectionRestore = {}
  const collaborationMode = readString(strategy.collaborationMode)
  const learningLevel = readString(strategy.learningLevel)
  const primaryRole = readString(strategy.primaryRole)
  const selectedRoles = readStringArray(strategy.selectedRoles)
  if (collaborationMode) selection.collaborationMode = collaborationMode
  if (learningLevel) selection.learningLevel = learningLevel
  if (primaryRole) selection.primaryRole = primaryRole
  if (selectedRoles.length > 0) selection.selectedRoles = selectedRoles
  return Object.keys(selection).length > 0 ? selection : null
}

function readStrategyId(contentJson: unknown): AgentChatStrategyId | null {
  const strategy = readStrategyRecord(contentJson)
  const strategyId = readString(strategy?.strategyId)
  return normalizeAgentChatStrategyId(strategyId)
}

function readContext(contentJson: unknown) {
  if (!isRecord(contentJson)) return null
  return isRecord(contentJson.context) ? contentJson.context : null
}

function readStrategyRecord(contentJson: unknown) {
  if (!isRecord(contentJson)) return null
  return isRecord(contentJson.strategy) ? contentJson.strategy : null
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

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null
}
