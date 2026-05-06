import { type ReactNode } from "react"
import { AgentChatContext } from "../context/agent-chat-context-value"
import { useAgentChatPage } from "../use-agent-chat-page"

export interface AgentChatProviderProps {
  children: ReactNode
}

export function AgentChatProvider({ children }: AgentChatProviderProps) {
  const chat = useAgentChatPage()

  return (
    <AgentChatContext.Provider value={chat}>
      {children}
    </AgentChatContext.Provider>
  )
}
