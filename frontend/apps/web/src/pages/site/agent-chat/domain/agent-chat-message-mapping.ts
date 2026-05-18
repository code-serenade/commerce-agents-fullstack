import type {
  AgentRuntimeEvent,
  ConversationDetailResponse,
  ConversationFileResponse,
  ConversationMessageResponse,
} from "@/api"
import { sanitizeAgentAnswerContent } from "./agent-chat-answer-sanitizer"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
  AgentChatMessage,
} from "../types"
import { readMessageContextSummary } from "./agent-chat-message-context-summary"
export {
  readConversationRuntimeEvents,
  readConversationRuntimeSelection,
  readConversationSelectedCampaignIds,
  readConversationSelectedProductIds,
  readConversationSelectedSourceRefs,
  readConversationStrategyId,
} from "./agent-chat-conversation-readers"
import { isAgentRuntimeEvent } from "./agent-chat-conversation-readers"

export interface AgentChatMessageFormatCopy {
  boundaryNotice: string
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
  return sanitizeAgentUiMessage(text, copy)
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

function sanitizeAgentUiMessage(content: string, copy: AgentChatMessageFormatCopy) {
  return sanitizeAgentAnswerContent(content, {
    boundaryNotice: copy.boundaryNotice,
  })
}
