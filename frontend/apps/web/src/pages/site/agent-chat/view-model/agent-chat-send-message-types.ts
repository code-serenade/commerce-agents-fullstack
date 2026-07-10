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
  AgentChatConversationItem,
  AgentChatMessage,
} from "../types"

export interface SendAgentChatMessageInput {
  attachments: AgentChatAttachmentItem[]
  content: string
  conversationId: string | null
  fallbackConversationTitle: string
  messageFormatCopy: AgentChatMessageFormatCopy
  onController: (controller: AbortController) => void
  onConversationCreated?: (conversationId: string) => void
  sessionIdRef: MutableRefObject<string>
  setConversationId: Dispatch<SetStateAction<string | null>>
  setConversations: Dispatch<SetStateAction<AgentChatConversationItem[]>>
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
