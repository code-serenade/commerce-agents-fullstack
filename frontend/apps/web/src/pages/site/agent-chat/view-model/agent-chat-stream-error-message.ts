import type { AgentChatMessageFormatCopy } from "../domain/agent-chat-message-mapping"
import { formatAgentUiMessage } from "../domain/agent-chat-message-mapping"
import { isAbortLikeStreamError } from "./agent-chat-stream-errors"

export function formatStreamErrorMessage(
  error: unknown,
  input: { messageFormatCopy: AgentChatMessageFormatCopy; tError: string }
) {
  if (isAbortLikeStreamError(error)) {
    return input.messageFormatCopy.streamTimedOut
  }
  return formatAgentUiMessage(
    error instanceof Error
      ? error.message
      : typeof error === "string"
        ? error
        : input.tError,
    input.messageFormatCopy
  )
}
