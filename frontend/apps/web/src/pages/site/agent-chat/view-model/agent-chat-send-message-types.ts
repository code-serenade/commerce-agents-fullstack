import type { Dispatch, MutableRefObject, SetStateAction } from "react"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import type { AgentChatMessageFormatCopy } from "../domain/agent-chat-message-mapping"
import type {
  AgentChatCampaignContextSelection,
  AgentChatProductContextSelection,
  AgentChatSourceContextSelection,
} from "../domain/agent-chat-context-selection"
import type { AgentChatRuntimeSelection } from "../domain/agent-runtime-options"
import type { AgentChatStrategy } from "../domain/agent-chat-strategy"
import type {
  AgentChatAttachmentItem,
  AgentChatMessage,
} from "../types"
import type {
  EnsureAgentChatConversation,
  PromoteAgentChatConversationTitle,
} from "./use-agent-chat-conversation-identity"

export interface SendAgentChatMessageInput {
  attachments: AgentChatAttachmentItem[]
  content: string
  conversationId: string | null
  conversationTitle?: string | null
  fallbackConversationTitle: string
  ensureConversation: EnsureAgentChatConversation
  messageFormatCopy: AgentChatMessageFormatCopy
  onController: (controller: AbortController) => void
  promoteConversationTitle: PromoteAgentChatConversationTitle
  sessionIdRef: MutableRefObject<string>
  setError: Dispatch<SetStateAction<string | null>>
  setInput: Dispatch<SetStateAction<string>>
  setIsStreaming: Dispatch<SetStateAction<boolean>>
  setMessages: Dispatch<SetStateAction<AgentChatMessage[]>>
  setRuntimeEvents: Dispatch<SetStateAction<AgentRuntimeEvent[]>>
  selectedCampaigns: AgentChatCampaignContextSelection[]
  selectedProducts: AgentChatProductContextSelection[]
  selectedSources: AgentChatSourceContextSelection[]
  runtimeSelection: AgentChatRuntimeSelection
  strategy: AgentChatStrategy
  tError: string
}
