import type {
  ConversationDetailResponse,
  ConversationFileResponse,
  ConversationMessageResponse,
} from "@/api"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import type { AgentChatRuntimeSelectionRestore } from "./agent-runtime-options"
import {
  DEFAULT_AGENT_CHAT_STRATEGY_ID,
  isAgentChatStrategyId,
  type AgentChatStrategyId,
} from "./agent-chat-strategy"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
  AgentChatMessage,
} from "../types"
import { readMessageContextSummary } from "./agent-chat-message-context-summary"

export interface AgentChatMessageFormatCopy {
  missingStructuredOutput: string
  streamTimedOut: string
}

export function mapConversationItem(
  conversation: {
    id: string
    title: string | null
    updatedAt: string
  },
  fallbackTitle: string
): AgentChatConversationItem {
  return {
    id: conversation.id,
    title: conversation.title || fallbackTitle,
    updatedAt: conversation.updatedAt,
  }
}

export function mapConversationMessages(
  detail: ConversationDetailResponse,
  fallback: AgentChatMessage,
  copy: AgentChatMessageFormatCopy
): AgentChatMessage[] {
  const messages = detail.messages
    .filter((message): message is ConversationMessageResponse & {
      role: "user" | "assistant"
    } => isVisibleMessage(message, copy))
    .map((message) => mapConversationMessage(message, copy))

  return messages.length > 0 ? messages : [fallback]
}

export function formatAgentUiMessage(
  content: string,
  copy: AgentChatMessageFormatCopy
) {
  const text = content.trim()
  if (!text || text === "missing") return ""
  if (/structured role output is missing/i.test(text)) {
    return copy.missingStructuredOutput
  }
  if (/Agent stream timed out without backend events/i.test(text)) {
    return copy.streamTimedOut
  }
  return sanitizeAgentUiMessage(text)
}

export function createConversationTitle(content: string) {
  return content.length > 40 ? `${content.slice(0, 40)}...` : content
}

export function createSessionId() {
  return `web-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`
}

export function createMessageId(prefix: string) {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`
}

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
  for (let index = detail.messages.length - 1; index >= 0; index -= 1) {
    const message = detail.messages[index]
    if (message?.role !== "user") continue
    const productIds = readProductIds(message.contentJson)
    if (productIds.length > 0) return productIds
  }
  return []
}

export function readConversationSelectedSourceRefs(
  detail: ConversationDetailResponse
): string[] {
  for (let index = detail.messages.length - 1; index >= 0; index -= 1) {
    const message = detail.messages[index]
    if (message?.role !== "user") continue
    const sourceRefs = readSourceRefs(message.contentJson)
    if (sourceRefs.length > 0) return sourceRefs
  }
  return []
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

export function mapConversationAttachments(
  detail: ConversationDetailResponse
): AgentChatAttachmentItem[] {
  return detail.files.map(mapConversationFile)
}

function mapConversationMessage(
  message: ConversationMessageResponse & { role: "user" | "assistant" },
  copy: AgentChatMessageFormatCopy
): AgentChatMessage {
  const contextSummary =
    message.role === "user"
      ? readMessageContextSummary(message.contentJson)
      : undefined

  return {
    id: message.id,
    role: message.role,
    content: formatAgentUiMessage(message.contentText ?? "", copy),
    runtimeEvents: readRuntimeEvents(message.contentJson),
    ...(contextSummary ? { contextSummary } : {}),
  }
}

function isVisibleMessage(
  message: ConversationMessageResponse,
  copy: AgentChatMessageFormatCopy
): message is ConversationMessageResponse & { role: "user" | "assistant" } {
  const content = formatAgentUiMessage(message.contentText ?? "", copy)
  return (
    (message.role === "user" || message.role === "assistant") &&
    Boolean(content)
  )
}

function readRuntimeEvents(contentJson: unknown): AgentRuntimeEvent[] {
  if (!isRecord(contentJson)) return []
  const runtimeEvents = contentJson.runtimeEvents
  if (!Array.isArray(runtimeEvents)) return []
  return runtimeEvents.filter(isAgentRuntimeEvent)
}

function readProductIds(contentJson: unknown): string[] {
  if (!isRecord(contentJson)) return []
  const context = contentJson.context
  if (!isRecord(context)) return []

  const productIds = readStringArray(context.productIds)
  const contextRefs = readStringArray(context.contextRefs)
    .map((item) => item.match(/^product:(.+)$/)?.[1])
    .filter((item): item is string => Boolean(item))

  return uniqueStrings([...productIds, ...contextRefs])
}

function readSourceRefs(contentJson: unknown): string[] {
  if (!isRecord(contentJson)) return []
  const context = contentJson.context
  if (!isRecord(context)) return []

  return readStringArray(context.contextRefs).filter((ref) =>
    ref.startsWith("conversation_file:")
  )
}

function readRuntimeSelection(
  contentJson: unknown
): AgentChatRuntimeSelectionRestore | null {
  const strategy = readStrategyRecord(contentJson)
  if (!strategy) return null

  const selection: AgentChatRuntimeSelectionRestore = {}
  const collaborationMode = readString(strategy.collaborationMode)
  const learningLevel = readString(strategy.learningLevel)
  const modelName = readString(strategy.modelName)
  const primaryRole = readString(strategy.primaryRole)
  const selectedRoles = readStringArray(strategy.selectedRoles)

  if (collaborationMode) selection.collaborationMode = collaborationMode
  if (learningLevel) selection.learningLevel = learningLevel
  if (modelName) selection.modelName = modelName
  if (primaryRole) selection.primaryRole = primaryRole
  if (selectedRoles.length > 0) selection.selectedRoles = selectedRoles

  return Object.keys(selection).length > 0 ? selection : null
}

function readStrategyId(contentJson: unknown): AgentChatStrategyId | null {
  const strategy = readStrategyRecord(contentJson)
  const strategyId = readString(strategy?.strategyId)
  return strategyId && isAgentChatStrategyId(strategyId) ? strategyId : null
}

function readStrategyRecord(contentJson: unknown) {
  if (!isRecord(contentJson)) return null
  const strategy = contentJson.strategy
  return isRecord(strategy) ? strategy : null
}

function mapConversationFile(
  file: ConversationFileResponse
): AgentChatAttachmentItem {
  return {
    id: file.id,
    fileId: file.id,
    name: file.fileName || file.objectKey.split("/").at(-1) || file.objectKey,
    readable: isReadableConversationFile(file),
    status: file.status === "failed" ? "failed" : "uploaded",
  }
}

function isAgentRuntimeEvent(value: unknown): value is AgentRuntimeEvent {
  return isRecord(value) && typeof value.type === "string"
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

function isReadableConversationFile(file: ConversationFileResponse) {
  const mimeType = file.mimeType?.toLowerCase() ?? ""
  if (mimeType.startsWith("text/")) return true
  if (
    [
      "application/json",
      "application/ld+json",
      "application/markdown",
    ].includes(mimeType)
  ) {
    return true
  }
  return /\.(csv|json|md|markdown|txt)$/i.test(file.fileName ?? file.objectKey)
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null
}

function sanitizeAgentUiMessage(content: string) {
  return content
    .split("\n")
    .map((line) => {
      const trimmedLine = line.trim()
      if (/^skill:[^\n]+/i.test(trimmedLine)) return ""
      return line.replace(/[;；]?\s*skill:[^\n。；;]+(?:[。.;])?/gi, "").trimEnd()
    })
    .filter((line) => Boolean(line.trim()))
    .join("\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim()
}
