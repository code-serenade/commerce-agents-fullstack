import { describe, expect, it, vi } from "vitest"
import { agentChatData } from "../agent-chat-data"
import { getAgentChatStrategy } from "../domain/agent-chat-strategy"
import { sendAgentChatMessage } from "./agent-chat-send-message"

vi.mock("../agent-chat-data", () => ({
  AGENT_ID: "web-agent",
  agentChatData: {
    addConversationMessage: vi.fn(),
    streamAgentSession: vi.fn(),
  },
}))

const titleKey = "title"

describe("sendAgentChatMessage title promotion", () => {
  it("promotes an existing fallback conversation title from the first user message", async () => {
    const ensureConversation = vi.fn()
    const promoteConversationTitle = vi.fn()
    vi.mocked(agentChatData.addConversationMessage).mockResolvedValue(
      {} as Awaited<ReturnType<typeof agentChatData.addConversationMessage>>
    )
    vi.mocked(agentChatData.streamAgentSession).mockImplementation(
      async (_body, handlers) => {
        handlers.onDelta("ok")
        handlers.onDone()
      }
    )

    await sendAgentChatMessage({
      content: "Analyze coffee category products.",
      attachments: [],
      conversationId: "conversation-existing",
      conversationTitle: "New chat",
      ensureConversation,
      fallbackConversationTitle: "New chat",
      messageFormatCopy: {
        boundaryNotice: "boundary notice",
        missingStructuredOutput: "missing structured output",
        streamTimedOut: "stream timed out",
      },
      onController: vi.fn(),
      promoteConversationTitle,
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
      sessionIdRef: { current: "session-existing" },
      setError: vi.fn(),
      setInput: vi.fn(),
      setIsStreaming: vi.fn(),
      setMessages: vi.fn(),
      setRuntimeEvents: vi.fn(),
      strategy: getAgentChatStrategy("execution"),
      tError: "error",
    })

    expect(ensureConversation).not.toHaveBeenCalled()
    expect(promoteConversationTitle).toHaveBeenCalledWith({
      conversationId: "conversation-existing",
      currentTitle: "New chat",
      [titleKey]: "Analyze coffee category products.",
    })
    expect(agentChatData.addConversationMessage).toHaveBeenCalledWith(
      "conversation-existing",
      expect.objectContaining({
        contentText: "Analyze coffee category products.",
        role: "user",
      })
    )
  })
})
