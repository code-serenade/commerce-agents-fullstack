import { describe, expect, it } from "vitest"
import {
  resetAgentChatConversationContext,
  startNewAgentConversation,
} from "./agent-chat-page-actions"
import type { AgentChatPageStateActions } from "./agent-chat-page-actions"

describe("agent chat page actions", () => {
  it("resets strategy and runtime context when starting a new conversation", () => {
    const calls: string[] = []
    const state = createPageState(calls)

    startNewAgentConversation(state)

    expect(calls).toContain("strategy:execution")
    expect(calls).toContain("runtime:reset")
    expect(calls).toContain("products:clear")
    expect(calls).toContain("sources:clear")
  })

  it("resets only the conversation-level runtime context", () => {
    const calls: string[] = []

    resetAgentChatConversationContext({
      resetRuntimeSelection: () => calls.push("runtime:reset"),
      restoreStrategyId: (strategyId) => calls.push(`strategy:${strategyId}`),
    })

    expect(calls).toEqual(["strategy:execution", "runtime:reset"])
  })
})

function createPageState(calls: string[]): AgentChatPageStateActions {
  const abortController = new AbortController()

  return {
    abortRef: { current: abortController },
    clearSelectedProducts: () => calls.push("products:clear"),
    clearSelectedSources: () => calls.push("sources:clear"),
    createWelcomeMessage: () => ({
      content: "",
      id: "welcome",
      role: "assistant",
    }),
    resetRuntimeSelection: () => calls.push("runtime:reset"),
    restoreStrategyId: (strategyId) => calls.push(`strategy:${strategyId}`),
    sessionIdRef: { current: "old-session" },
    setConversationId: () => undefined,
    setConversations: () => undefined,
    setError: () => undefined,
    setInput: () => undefined,
    setIsStreaming: () => undefined,
    setMessages: () => undefined,
    setRuntimeEvents: () => undefined,
  }
}
