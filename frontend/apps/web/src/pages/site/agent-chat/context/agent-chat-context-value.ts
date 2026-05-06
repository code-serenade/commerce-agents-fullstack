import { createContext } from "react"
import { useAgentChatPage } from "../use-agent-chat-page"

export type AgentChatContextValue = ReturnType<typeof useAgentChatPage>

export const AgentChatContext = createContext<AgentChatContextValue | null>(null)
