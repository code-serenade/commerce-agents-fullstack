import { useContext } from "react"
import { AgentChatContext } from "./agent-chat-context-value"

export function useAgentChatContext() {
  const context = useContext(AgentChatContext)
  if (!context) {
    throw new Error("useAgentChatContext must be used within AgentChatProvider")
  }
  return context
}
