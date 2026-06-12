import { useCallback } from "react"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import type { AgentChatRuntimeSelection } from "../domain/agent-runtime-options"

interface UseAgentChatStrategyRuntimeInput {
  runtimeSelection: AgentChatRuntimeSelection
  setRoutingMode: (value: string) => void
  setStrategyId: (value: AgentChatStrategyId) => void
}

export function useAgentChatStrategyRuntime({
  runtimeSelection,
  setRoutingMode,
  setStrategyId,
}: UseAgentChatStrategyRuntimeInput) {
  return useCallback((nextStrategyId: AgentChatStrategyId) => {
    setStrategyId(nextStrategyId)
    if (nextStrategyId === "quick") {
      setRoutingMode("fast")
      return
    }
    if (runtimeSelection.routingMode === "fast") {
      setRoutingMode("commander")
    }
  }, [runtimeSelection.routingMode, setRoutingMode, setStrategyId])
}
