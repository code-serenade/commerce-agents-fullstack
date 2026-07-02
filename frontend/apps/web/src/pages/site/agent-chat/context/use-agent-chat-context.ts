import { useContext } from "react"
import { AgentChatContext } from "./agent-chat-context-value"

export function useOptionalAgentChatContext() {
  return useContext(AgentChatContext)
}

export function useAgentChatContext() {
  const context = useOptionalAgentChatContext()
  if (!context) {
    throw new Error("useAgentChatContext must be used within AgentChatProvider")
  }
  return context
}
