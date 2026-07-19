import {
  useCallback,
  useRef,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react"
import { useSelector } from "react-redux"
import type { RootState } from "@/store"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import { createSessionId } from "../domain/agent-chat-message-mapping"
import {
  DEFAULT_AGENT_CHAT_STRATEGY_ID,
  type AgentChatStrategyId,
} from "../domain/agent-chat-strategy"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
  AgentChatMessage,
} from "../types"

export function useAgentChatSessionState(welcomeMessage: string) {
  const user = useSelector((state: RootState) => state.auth.user)
  const [input, setInput] = useState("")
  const [strategyId, setStrategyId] = useState<AgentChatStrategyId>(
    DEFAULT_AGENT_CHAT_STRATEGY_ID
  )
  const [messages, setMessages] = useState<AgentChatMessage[]>(() => [
    {
      id: "welcome",
      role: "assistant",
      content: welcomeMessage,
    },
  ])
  const [conversationId, setConversationId] = useState<string | null>(null)
  const [conversations, setConversations] = useState<
    AgentChatConversationItem[]
  >([])
  const [attachments, setAttachments] = useState<AgentChatAttachmentItem[]>([])
  const [isStreaming, setIsStreaming] = useState(false)
  const [runtimeEvents, setRuntimeEvents] = useState<AgentRuntimeEvent[]>([])
  const [error, setError] = useState<string | null>(null)
  const abortRef = useRef<AbortController | null>(null)
  const sessionIdRef = useRef(createSessionId())
  const conversationIdRef = useRef(conversationId)
  const setActiveConversationId = useCallback<
    Dispatch<SetStateAction<string | null>>
  >((value) => {
    setConversationId((current) => {
      const next = typeof value === "function" ? value(current) : value
      conversationIdRef.current = next
      return next
    })
  }, [])

  return {
    abortRef,
    attachments,
    conversationId,
    conversationIdRef,
    conversations,
    error,
    input,
    isLoggedIn: Boolean(user),
    isStreaming,
    messages,
    runtimeEvents,
    sessionIdRef,
    setActiveConversationId,
    setAttachments,
    setConversations,
    setError,
    setInput,
    setIsStreaming,
    setMessages,
    setRuntimeEvents,
    setStrategyId,
    strategyId,
    user,
  }
}
