import { describe, expect, it, vi } from "vitest"
import { agentChatData } from "../agent-chat-data"
import { getAgentChatStrategy } from "../domain/agent-chat-strategy"
import { sendAgentChatMessage } from "./agent-chat-send-message"

const imageTitle = "shelf.png"

vi.mock("../agent-chat-data", () => ({
  AGENT_ID: "web-agent",
  agentChatData: {
    addConversationMessage: vi.fn(),
    createConversation: vi.fn(),
    streamAgentSession: vi.fn(),
  },
}))

describe("sendAgentChatMessage image attachments", () => {
  it("sends selected uploaded images as vision attachments", async () => {
    const addConversationMessage = vi.mocked(agentChatData.addConversationMessage)
    const createConversation = vi.mocked(agentChatData.createConversation)
    const streamAgentSession = vi.mocked(agentChatData.streamAgentSession)

    createConversation.mockResolvedValue({
      conversation: {
        id: "conversation-image",
        ["title"]: "",
        updatedAt: "2026-05-16T00:00:00.000Z",
      },
    } as Awaited<ReturnType<typeof agentChatData.createConversation>>)
    addConversationMessage.mockResolvedValue({} as Awaited<
      ReturnType<typeof agentChatData.addConversationMessage>
    >)
    streamAgentSession.mockImplementation(async (_body, handlers) => {
      handlers.onDelta("image ok")
      handlers.onDone()
    })

    await sendAgentChatMessage({
      attachments: [
        {
          fileId: "file-image",
          id: "local-image",
          imageDataUrl: "data:image/png;base64,aW1hZ2U=",
          imageDetail: "auto",
          kind: "image",
          name: imageTitle,
          readable: true,
          status: "uploaded",
        },
      ],
      content: "Read this shelf image.",
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
          id: "file-image",
          kind: "conversation_file",
          readable: true,
          ref: "conversation_file:file-image",
          status: "uploaded",
          title: imageTitle,
          visionReady: true,
        },
      ],
      sessionIdRef: { current: "session-image" },
      setConversationId: vi.fn(),
      setConversations: vi.fn(),
      setError: vi.fn(),
      setInput: vi.fn(),
      setIsStreaming: vi.fn(),
      setMessages: vi.fn(),
      setRuntimeEvents: vi.fn(),
      strategy: getAgentChatStrategy("execution"),
      tError: "error",
    })

    expect(streamAgentSession).toHaveBeenCalledWith(
      expect.objectContaining({
        metadata: expect.objectContaining({
          attachments: {
            images: [
              {
                detail: "auto",
                sourceRef: "conversation_file:file-image",
                url: "data:image/png;base64,aW1hZ2U=",
              },
            ],
          },
          context: expect.objectContaining({
            contextRefs: ["conversation_file:file-image"],
          }),
        }),
      }),
      expect.any(Object)
    )
  })

  it("does not send uploaded images until the user selects them for this turn", async () => {
    const addConversationMessage = vi.mocked(agentChatData.addConversationMessage)
    const createConversation = vi.mocked(agentChatData.createConversation)
    const streamAgentSession = vi.mocked(agentChatData.streamAgentSession)

    createConversation.mockResolvedValue({
      conversation: {
        id: "conversation-image-unselected",
        ["title"]: "",
        updatedAt: "2026-05-16T00:00:00.000Z",
      },
    } as Awaited<ReturnType<typeof agentChatData.createConversation>>)
    addConversationMessage.mockResolvedValue({} as Awaited<
      ReturnType<typeof agentChatData.addConversationMessage>
    >)
    streamAgentSession.mockImplementation(async (_body, handlers) => {
      handlers.onDelta("image skipped")
      handlers.onDone()
    })

    await sendAgentChatMessage({
      attachments: [
        {
          fileId: "file-image-unselected",
          id: "local-image-unselected",
          imageDataUrl: "data:image/png;base64,aW1hZ2U=",
          imageDetail: "auto",
          kind: "image",
          name: imageTitle,
          readable: true,
          status: "uploaded",
        },
      ],
      content: "Answer without reading the image.",
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
      selectedSources: [],
      sessionIdRef: { current: "session-image-unselected" },
      setConversationId: vi.fn(),
      setConversations: vi.fn(),
      setError: vi.fn(),
      setInput: vi.fn(),
      setIsStreaming: vi.fn(),
      setMessages: vi.fn(),
      setRuntimeEvents: vi.fn(),
      strategy: getAgentChatStrategy("execution"),
      tError: "error",
    })

    expect(streamAgentSession).toHaveBeenCalledWith(
      expect.objectContaining({
        metadata: expect.not.objectContaining({
          attachments: expect.anything(),
        }),
      }),
      expect.any(Object)
    )
    expect(streamAgentSession).toHaveBeenCalledWith(
      expect.objectContaining({
        metadata: expect.objectContaining({
          context: expect.objectContaining({
            contextRefs: [],
          }),
        }),
      }),
      expect.any(Object)
    )
  })
})
