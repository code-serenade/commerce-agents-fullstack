import { beforeEach, describe, expect, it, vi } from "vitest"
import { agentChatData } from "../agent-chat-data"
import { getAgentChatStrategy } from "../domain/agent-chat-strategy"
import type { AgentChatMessage } from "../types"
import { sendAgentChatMessage } from "./agent-chat-send-message"
import type { SendAgentChatMessageInput } from "./agent-chat-send-message-types"

vi.mock("../agent-chat-data", () => ({
  AGENT_ID: "web-agent",
  agentChatData: {
    addConversationMessage: vi.fn(),
    createConversation: vi.fn(),
    streamAgentSession: vi.fn(),
  },
}))

describe("sendAgentChatMessage late stream errors", () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it("ignores a late stream error after the assistant answer is done", async () => {
    const addConversationMessage = mockConversation("conversation-f")
    const messages: AgentChatMessage[] = []
    const errors: Array<string | null> = []
    const streamingStates: boolean[] = []

    vi.mocked(agentChatData.streamAgentSession).mockImplementation(
      async (_body, handlers) => {
        handlers.onDelta("completed answer")
        handlers.onDone()
        handlers.onError("Agent stream ended without output.")
      }
    )

    await sendAgentChatMessage(createInput("session-f", { errors, messages, streamingStates }))

    const assistant = messages.find((message) => message.role === "assistant")
    expect(errors).toEqual([null])
    expect(streamingStates).toContain(false)
    expect(assistant?.content).toContain("completed answer")
    expect(assistant?.status).toBeUndefined()
    expect(addConversationMessage).toHaveBeenLastCalledWith(
      "conversation-f",
      expect.objectContaining({
        contentText: expect.stringContaining("completed answer"),
        role: "assistant",
      })
    )
  })

  it("keeps a completed answer when the transport rejects after done", async () => {
    const addConversationMessage = mockConversation("conversation-g")
    const messages: AgentChatMessage[] = []
    const errors: Array<string | null> = []

    vi.mocked(agentChatData.streamAgentSession).mockImplementation(
      async (_body, handlers) => {
        handlers.onDelta("saved answer")
        handlers.onDone()
        throw new Error("Agent stream ended without output.")
      }
    )

    await sendAgentChatMessage(createInput("session-g", { errors, messages }))

    const assistant = messages.find((message) => message.role === "assistant")
    expect(errors).toEqual([null])
    expect(assistant?.content).toContain("saved answer")
    expect(assistant?.status).toBeUndefined()
    expect(addConversationMessage).toHaveBeenLastCalledWith(
      "conversation-g",
      expect.objectContaining({
        contentText: expect.stringContaining("saved answer"),
        role: "assistant",
      })
    )
  })
})

function mockConversation(id: string) {
  vi.mocked(agentChatData.createConversation).mockResolvedValue({
    conversation: { id, ["title"]: "", updatedAt: "2026-05-16T00:00:00.000Z" },
  } as Awaited<ReturnType<typeof agentChatData.createConversation>>)
  vi.mocked(agentChatData.addConversationMessage).mockResolvedValue({} as Awaited<
    ReturnType<typeof agentChatData.addConversationMessage>
  >)
  return vi.mocked(agentChatData.addConversationMessage)
}

function createInput(
  sessionId: string,
  state: {
    errors: Array<string | null>
    messages: AgentChatMessage[]
    streamingStates?: boolean[]
  }
): SendAgentChatMessageInput {
  return {
    content: "Analyze live commerce conversion.",
    attachments: [],
    conversationId: null,
    fallbackConversationTitle: "New chat",
    messageFormatCopy: {
      boundaryNotice: "boundary notice",
      missingStructuredOutput: "missing structured output",
      streamTimedOut: "stream timed out",
    },
    onController: vi.fn(),
    runtimeSelection: {
      collaborationMode: "manual",
      learningLevel: "higher_vocational",
      primaryRole: "ops",
      routingMode: "commander",
      selectedRoles: ["ops", "data"],
    },
    selectedProducts: [],
    selectedCampaigns: [],
    selectedSources: [],
    sessionIdRef: { current: sessionId },
    setConversationId: vi.fn(),
    setConversations: vi.fn(),
    setError: (value) => {
      state.errors.push(
        typeof value === "function" ? value(state.errors.at(-1) ?? null) : value
      )
    },
    setInput: vi.fn(),
    setIsStreaming: (value) => {
      state.streamingStates?.push(
        typeof value === "function"
          ? value(state.streamingStates.at(-1) ?? false)
          : value
      )
    },
    setMessages: (updater) => {
      if (typeof updater === "function") {
        state.messages.splice(0, state.messages.length, ...updater(state.messages))
      }
    },
    setRuntimeEvents: vi.fn(),
    strategy: getAgentChatStrategy("execution"),
    tError: "error",
  }
}
