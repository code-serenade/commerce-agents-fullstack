import { describe, expect, it, vi } from "vitest"
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
const readableFileTitle = "readable.pdf"
const pendingFileTitle = "pending.docx"

describe("sendAgentChatMessage source context", () => {
  it("persists selected source readiness for the user message context", async () => {
    const addConversationMessage = vi.mocked(agentChatData.addConversationMessage)
    const createConversation = vi.mocked(agentChatData.createConversation)
    const streamAgentSession = vi.mocked(agentChatData.streamAgentSession)
    const messages: AgentChatMessage[] = []

    createConversation.mockResolvedValue({
      conversation: {
        id: "conversation-sources",
        [titleKey]: "",
        updatedAt: "2026-05-16T00:00:00.000Z",
      },
    } as Awaited<ReturnType<typeof agentChatData.createConversation>>)
    addConversationMessage.mockResolvedValue({} as Awaited<
      ReturnType<typeof agentChatData.addConversationMessage>
    >)
    streamAgentSession.mockImplementation(async (_body, handlers) => {
      handlers.onDelta("ok")
      handlers.onDone()
    })

    await sendAgentChatMessage({
      content: "Analyze the selected materials.",
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
        selectedRoles: ["ops"],
      },
      selectedProducts: [],
      selectedCampaigns: [],
      selectedSources: [
        {
          id: "file-readable",
          kind: "conversation_file",
          readable: true,
          ref: "conversation_file:file-readable",
          status: "uploaded",
          title: readableFileTitle,
        },
        {
          id: "file-pending",
          kind: "conversation_file",
          readable: false,
          ref: "conversation_file:file-pending",
          status: "pending",
          title: pendingFileTitle,
        },
      ],
      sessionIdRef: { current: "session-sources" },
      setConversationId: vi.fn(),
      setConversations: vi.fn(),
      setError: vi.fn(),
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

    const expectedSources = [
      {
        readiness: "readable",
        ref: "conversation_file:file-readable",
        title: readableFileTitle,
      },
      {
        readiness: "pending",
        ref: "conversation_file:file-pending",
        title: pendingFileTitle,
      },
    ]

    expect(addConversationMessage).toHaveBeenCalledWith(
      "conversation-sources",
      expect.objectContaining({
        contentJson: expect.objectContaining({
          context: expect.objectContaining({
            contextRefs: [
              "conversation_file:file-readable",
              "conversation_file:file-pending",
            ],
            sources: expectedSources,
          }),
        }),
      })
    )
    expect(messages[0]?.contextSummary?.sources).toEqual([
      {
        id: "conversation_file:file-readable",
        label: readableFileTitle,
        readiness: "readable",
      },
      {
        id: "conversation_file:file-pending",
        label: pendingFileTitle,
        readiness: "pending",
      },
    ])
  })
})
