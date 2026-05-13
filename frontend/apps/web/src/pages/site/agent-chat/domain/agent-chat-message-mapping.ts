import type {
  ConversationDetailResponse,
  ConversationMessageResponse,
} from "@/api"
import type { AgentChatConversationItem, AgentChatMessage } from "../types"

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
    .map((message) => ({
      id: message.id,
      role: message.role,
      content: formatAgentUiMessage(message.contentText ?? "", copy),
    }))

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
  return text
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
