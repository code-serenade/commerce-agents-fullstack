import { describe, expect, it } from "vitest"
import {
  resetAgentChatConversationContext,
  startNewAgentConversation,
  stopAgentStreaming,
} from "./agent-chat-page-actions"
import type { AgentChatPageStateActions } from "./agent-chat-page-actions"
import type { AgentChatMessage } from "../types"

describe("agent chat page actions", () => {
  it("resets strategy and runtime context when starting a new conversation", () => {
    const calls: string[] = []
    const state = createPageState(calls)

    startNewAgentConversation(state)

    expect(calls).toContain("strategy:learning")
    expect(calls).toContain("runtime:reset")
    expect(calls).toContain("campaigns:clear")
    expect(calls).toContain("products:clear")
    expect(calls).toContain("sources:clear")
  })

  it("does not append placeholder history when starting a new conversation", () => {
    const calls: string[] = []
    const state = createPageState(calls)
    let conversationsWereMutated = false
    state.setConversations = () => {
      conversationsWereMutated = true
    }

    startNewAgentConversation(state)

    expect(conversationsWereMutated).toBe(false)
    expect(calls).toContain("strategy:learning")
  })

  it("resets only the conversation-level runtime context", () => {
    const calls: string[] = []

    resetAgentChatConversationContext({
      resetRuntimeSelection: () => calls.push("runtime:reset"),
      restoreStrategyId: (strategyId) => calls.push(`strategy:${strategyId}`),
    })

    expect(calls).toEqual(["strategy:learning", "runtime:reset"])
  })

  it("marks partial streaming answers as stopped when stopping generation", () => {
    const calls: string[] = []
    const state = createPageState(calls)
    let streaming = true
    const messages: AgentChatMessage[] = [
      {
        content: "partial answer",
        id: "assistant-1",
        role: "assistant" as const,
        status: "streaming" as const,
      },
    ]

    state.setIsStreaming = (value) => {
      streaming = typeof value === "function" ? value(streaming) : value
    }
    state.setMessages = (updater) => {
      if (typeof updater === "function") {
        messages.splice(0, messages.length, ...updater(messages))
      }
    }

    stopAgentStreaming(state)

    expect(state.abortRef.current).toBeNull()
    expect(streaming).toBe(false)
    expect(messages[0]?.status).toBe("stopped")
  })
})

function createPageState(calls: string[]): AgentChatPageStateActions {
  const abortController = new AbortController()

  return {
    abortRef: { current: abortController },
    clearSelectedCampaigns: () => calls.push("campaigns:clear"),
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
