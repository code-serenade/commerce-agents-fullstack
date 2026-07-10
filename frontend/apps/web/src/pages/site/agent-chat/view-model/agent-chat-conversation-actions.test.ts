import { describe, expect, it, vi } from "vitest"
import { agentChatData } from "../agent-chat-data"
import {
  loadAgentConversationById,
  loadAgentConversationList,
} from "./agent-chat-conversation-actions"
import type { AgentChatMessage } from "../types"

vi.mock("../agent-chat-data", () => ({
  agentChatData: {
    getConversation: vi.fn(),
    listConversations: vi.fn(),
  },
}))

describe("agent chat conversation actions", () => {
  it("loads the conversation list without selecting the first conversation", async () => {
    const state = createState()
    vi.mocked(agentChatData.listConversations).mockResolvedValue([
      createConversation("conversation-latest", "Latest"),
      createConversation("conversation-older", "Older"),
    ] as Awaited<ReturnType<typeof agentChatData.listConversations>>)

    await loadAgentConversationList(state, () => false)

    expect(state.setConversations).toHaveBeenCalledTimes(1)
    expect(state.setConversationId).not.toHaveBeenCalled()
    expect(agentChatData.getConversation).not.toHaveBeenCalled()
  })

  it("loads the route conversation id as the active conversation", async () => {
    const state = createState()
    vi.mocked(agentChatData.getConversation).mockResolvedValue(
      createConversationDetail("conversation-route")
    )

    await loadAgentConversationById("conversation-route", state, () => false)

    expect(agentChatData.getConversation).toHaveBeenCalledWith(
      "conversation-route"
    )
    expect(state.setConversationId).toHaveBeenCalledWith("conversation-route")
    expect(state.sessionIdRef.current).toBe("conversation-route")
  })
})

function createState() {
  const welcomeMessage: AgentChatMessage = {
    id: "welcome",
    role: "assistant",
    content: "Welcome",
  }

  return {
    createWelcomeMessage: () => welcomeMessage,
    fallbackConversationTitle: "New chat",
    messageFormatCopy: {
      boundaryNotice: "boundary notice",
      missingStructuredOutput: "missing structured output",
      streamTimedOut: "stream timed out",
    },
    sessionIdRef: { current: "session-initial" },
    setConversationId: vi.fn(),
    setConversations: vi.fn(),
    setError: vi.fn(),
    setInput: vi.fn(),
    setMessages: vi.fn(),
    setRuntimeEvents: vi.fn(),
    tError: "error",
  }
}

function createConversation(id: string, title: string) {
  return {
    id,
    title,
    updatedAt: "2026-07-10T00:00:00.000Z",
  }
}

function createConversationDetail(id: string) {
  return ({
    conversation: createConversation(id, "Route conversation"),
    files: [],
    messages: [],
  } as unknown) as Awaited<ReturnType<typeof agentChatData.getConversation>>
}
