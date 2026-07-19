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

const DEFAULT_TITLE = ["New", "chat"].join(" ")
const LEGACY_ZH_DEFAULT_TITLE = String.fromCharCode(26032, 24314, 23545, 35805)
const MANUAL_TITLE = "conversation-title"
const MESSAGE_CONTENT = "message-content"

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

  it("hides empty default-title conversations and names contentful ones", async () => {
    const state = createState()
    vi.mocked(agentChatData.listConversations).mockResolvedValue([
      createConversation("conversation-empty", DEFAULT_TITLE),
      createConversation("conversation-with-message", DEFAULT_TITLE),
      createConversation("conversation-titled", MANUAL_TITLE),
    ] as Awaited<ReturnType<typeof agentChatData.listConversations>>)
    vi.mocked(agentChatData.getConversation).mockImplementation(
      async (conversationId) => {
        if (conversationId === "conversation-with-message") {
          return createConversationDetail(conversationId, {
            messages: [
              {
                id: "message-1",
                role: "user",
                contentText: MESSAGE_CONTENT,
              },
            ],
          })
        }

        return createConversationDetail(conversationId)
      }
    )

    const conversations = await loadAgentConversationList(state, () => false)

    expect(conversations.map((item) => item.id)).toEqual([
      "conversation-with-message",
      "conversation-titled",
    ])
    expect(state.setConversations).toHaveBeenCalledWith([
      {
        id: "conversation-with-message",
        title: MESSAGE_CONTENT,
        updatedAt: "2026-07-10T00:00:00.000Z",
      },
      {
        id: "conversation-titled",
        title: MANUAL_TITLE,
        updatedAt: "2026-07-10T00:00:00.000Z",
      },
    ])
    expect(agentChatData.getConversation).toHaveBeenCalledTimes(2)
    expect(agentChatData.getConversation).not.toHaveBeenCalledWith(
      "conversation-titled"
    )
  })

  it("keeps default-title conversations when detail loading fails", async () => {
    const state = createState()
    vi.mocked(agentChatData.listConversations).mockResolvedValue([
      createConversation("conversation-fallback", LEGACY_ZH_DEFAULT_TITLE),
    ] as Awaited<ReturnType<typeof agentChatData.listConversations>>)
    vi.mocked(agentChatData.getConversation).mockRejectedValue(
      new Error("network failed")
    )

    const conversations = await loadAgentConversationList(state, () => false)

    expect(conversations.map((item) => item.id)).toEqual([
      "conversation-fallback",
    ])
    expect(state.setConversations).toHaveBeenCalledWith([
      {
        id: "conversation-fallback",
        title: LEGACY_ZH_DEFAULT_TITLE,
        updatedAt: "2026-07-10T00:00:00.000Z",
      },
    ])
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
    fallbackConversationTitle: DEFAULT_TITLE,
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

function createConversationDetail(
  id: string,
  overrides?: {
    messages?: Array<{
      id: string
      role: "user" | "assistant" | "system" | "tool"
      contentText: string | null
    }>
    files?: unknown[]
  }
) {
  return ({
    conversation: createConversation(id, "Route conversation"),
    files: overrides?.files ?? [],
    messages: overrides?.messages ?? [],
  } as unknown) as Awaited<ReturnType<typeof agentChatData.getConversation>>
}
