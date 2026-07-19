import { beforeEach, describe, expect, it, vi } from "vitest"
import { agentChatData } from "../agent-chat-data"
import { getAgentChatStrategy } from "../domain/agent-chat-strategy"
import type { AgentChatMessage } from "../types"
import { sendAgentChatMessage } from "./agent-chat-send-message"

vi.mock("../agent-chat-data", () => ({
  AGENT_ID: "web-agent",
  agentChatData: {
    addConversationMessage: vi.fn(),
    createConversation: vi.fn(),
    streamAgentSession: vi.fn(),
  },
}))

const titleKey = "title"

describe("sendAgentChatMessage stream abort fallback", () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it("keeps an existing answer as completed when the stream closes after output", async () => {
    const addConversationMessage = vi.mocked(agentChatData.addConversationMessage)
    const createConversation = vi.mocked(agentChatData.createConversation)
    const streamAgentSession = vi.mocked(agentChatData.streamAgentSession)
    const messages: AgentChatMessage[] = []
    const errors: Array<string | null> = []
    const streamingStates: boolean[] = []

    createConversation.mockResolvedValue({
      conversation: {
        id: "conversation-d",
        ["title"]: "",
        updatedAt: "2026-05-16T00:00:00.000Z",
      },
    } as Awaited<ReturnType<typeof agentChatData.createConversation>>)
    addConversationMessage.mockResolvedValue({} as Awaited<
      ReturnType<typeof agentChatData.addConversationMessage>
    >)
    streamAgentSession.mockImplementation(async (_body, handlers) => {
      handlers.onDelta("partial but useful answer")
      handlers.onError("connection closed before final done event")
    })

    await sendAgentChatMessage({
      content: "Analyze live commerce conversion.",
      attachments: [],
      conversationId: null,
      ensureConversation: async () =>
        (await createConversation({ [titleKey]: "New chat", agentId: "web-agent" }))
          .conversation.id,
      fallbackConversationTitle: "New chat",
      messageFormatCopy: {
        boundaryNotice: "boundary notice",
        missingStructuredOutput: "missing structured output",
        streamTimedOut: "stream timed out",
      },
      onController: vi.fn(),
      promoteConversationTitle: vi.fn(),
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
      sessionIdRef: { current: "session-d" },
      setError: (value) => {
        errors.push(
          typeof value === "function" ? value(errors.at(-1) ?? null) : value
        )
      },
      setInput: vi.fn(),
      setIsStreaming: (value) => {
        streamingStates.push(
          typeof value === "function"
            ? value(streamingStates.at(-1) ?? false)
            : value
        )
      },
      setMessages: (updater) => {
        if (typeof updater === "function") {
          messages.splice(0, messages.length, ...updater(messages))
        }
      },
      setRuntimeEvents: vi.fn(),
      strategy: getAgentChatStrategy("execution"),
      tError: "error",
    })

    const assistant = messages.find((message) => message.role === "assistant")
    expect(errors).toEqual([null])
    expect(streamingStates).toContain(false)
    expect(assistant?.content).toContain("partial but useful answer")
    expect(assistant?.status).toBeUndefined()
    expect(addConversationMessage).toHaveBeenLastCalledWith(
      "conversation-d",
      expect.objectContaining({
        contentText: expect.stringContaining("partial but useful answer"),
        role: "assistant",
      })
    )
  })

  it("normalizes abort errors when the stream closes before any answer", async () => {
    const addConversationMessage = vi.mocked(agentChatData.addConversationMessage)
    const createConversation = vi.mocked(agentChatData.createConversation)
    const streamAgentSession = vi.mocked(agentChatData.streamAgentSession)
    const messages: AgentChatMessage[] = []
    const errors: Array<string | null> = []

    createConversation.mockResolvedValue({
      conversation: {
        id: "conversation-e",
        ["title"]: "",
        updatedAt: "2026-05-16T00:00:00.000Z",
      },
    } as Awaited<ReturnType<typeof agentChatData.createConversation>>)
    addConversationMessage.mockResolvedValue({} as Awaited<
      ReturnType<typeof agentChatData.addConversationMessage>
    >)
    streamAgentSession.mockImplementation(async (_body, handlers) => {
      handlers.onError("signal is aborted without reason")
    })

    await sendAgentChatMessage({
      content: "Analyze live commerce conversion.",
      attachments: [],
      conversationId: null,
      ensureConversation: async () =>
        (await createConversation({ [titleKey]: "New chat", agentId: "web-agent" }))
          .conversation.id,
      fallbackConversationTitle: "New chat",
      messageFormatCopy: {
        boundaryNotice: "boundary notice",
        missingStructuredOutput: "missing structured output",
        streamTimedOut: "stream timed out",
      },
      onController: vi.fn(),
      promoteConversationTitle: vi.fn(),
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
      sessionIdRef: { current: "session-e" },
      setError: (value) => {
        errors.push(
          typeof value === "function" ? value(errors.at(-1) ?? null) : value
        )
      },
      setInput: vi.fn(),
      setIsStreaming: vi.fn(),
      setMessages: (updater) => {
        if (typeof updater === "function") {
          messages.splice(0, messages.length, ...updater(messages))
        }
      },
      setRuntimeEvents: vi.fn(),
      strategy: getAgentChatStrategy("execution"),
      tError: "error",
    })

    const assistant = messages.find((message) => message.role === "assistant")
    expect(errors).toEqual([null, "stream timed out"])
    expect(assistant?.content).toBe("stream timed out")
    expect(assistant?.content).not.toContain("signal is aborted")
    expect(assistant?.status).toBe("error")
  })
})
