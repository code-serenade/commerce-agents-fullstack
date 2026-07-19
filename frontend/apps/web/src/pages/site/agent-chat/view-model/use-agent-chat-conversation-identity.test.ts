import type { Dispatch, SetStateAction } from "react"
import { act, renderHook } from "@testing-library/react"
import { beforeEach, describe, expect, it, vi } from "vitest"
import type { ConversationDetailResponse, ConversationResponse } from "@/api"
import { agentChatData } from "../agent-chat-data"
import type { AgentChatConversationItem } from "../types"
import { useAgentChatConversationIdentity } from "./use-agent-chat-conversation-identity"

vi.mock("../agent-chat-data", () => ({
  AGENT_ID: "web-agent",
  agentChatData: {
    createConversation: vi.fn(),
    updateConversation: vi.fn(),
  },
}))

const titleKey = "title"

describe("useAgentChatConversationIdentity", () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it("shares one created conversation across concurrent first-message and upload paths", async () => {
    const creation = deferred<ConversationDetailResponse>()
    vi.mocked(agentChatData.createConversation).mockReturnValue(creation.promise)
    const state = createIdentityState()
    const { result } = renderHook(() =>
      useAgentChatConversationIdentity(state.input)
    )

    const firstMessage = result.current.ensureConversation({
      [titleKey]: "Analyze coffee products",
    })
    const upload = result.current.ensureConversation({ [titleKey]: "New chat" })

    expect(agentChatData.createConversation).toHaveBeenCalledTimes(1)

    await act(async () => {
      creation.resolve(
        createConversationDetail("conversation-one", "Analyze coffee products")
      )
      await Promise.all([firstMessage, upload])
    })

    await expect(firstMessage).resolves.toBe("conversation-one")
    await expect(upload).resolves.toBe("conversation-one")
    expect(state.conversationIdRef.current).toBe("conversation-one")
    expect(state.conversations.map((item) => item.id)).toEqual([
      "conversation-one",
    ])
  })

  it("shares one created conversation across concurrent attachment/source operations", async () => {
    const creation = deferred<ConversationDetailResponse>()
    vi.mocked(agentChatData.createConversation).mockReturnValue(creation.promise)
    const state = createIdentityState()
    const { result } = renderHook(() =>
      useAgentChatConversationIdentity(state.input)
    )

    const attachment = result.current.ensureConversation({
      [titleKey]: "New chat",
    })
    const source = result.current.ensureConversation({ [titleKey]: "New chat" })

    expect(agentChatData.createConversation).toHaveBeenCalledTimes(1)

    await act(async () => {
      creation.resolve(
        createConversationDetail("conversation-shared", "New chat")
      )
      await Promise.all([attachment, source])
    })

    await expect(attachment).resolves.toBe("conversation-shared")
    await expect(source).resolves.toBe("conversation-shared")
    expect(state.conversations).toHaveLength(1)
  })

  it("promotes a fallback title to the first user message title", async () => {
    vi.mocked(agentChatData.createConversation).mockResolvedValue(
      createConversationDetail("conversation-title", "New chat")
    )
    vi.mocked(agentChatData.updateConversation).mockResolvedValue(
      createConversation("conversation-title", "Analyze coffee products")
    )
    const state = createIdentityState()
    const { result } = renderHook(() =>
      useAgentChatConversationIdentity(state.input)
    )

    await act(async () => {
      const conversationId = await result.current.ensureConversation({
        [titleKey]: "New chat",
      })
      await result.current.promoteConversationTitle({
        conversationId,
        [titleKey]: "Analyze coffee products",
      })
    })

    expect(agentChatData.updateConversation).toHaveBeenCalledWith(
      "conversation-title",
      { [titleKey]: "Analyze coffee products" }
    )
    expect(state.conversations[0]).toMatchObject({
      id: "conversation-title",
      [titleKey]: "Analyze coffee products",
    })
  })

  it("promotes a restored fallback title from history on the first sent message", async () => {
    vi.mocked(agentChatData.updateConversation).mockResolvedValue(
      createConversation("conversation-restored", "Analyze restored materials")
    )
    const state = createIdentityState({
      conversationId: "conversation-restored",
      conversations: [
        {
          id: "conversation-restored",
          [titleKey]: "New chat",
          updatedAt: "2026-07-16T00:00:00.000Z",
        },
      ],
    })
    const { result } = renderHook(() =>
      useAgentChatConversationIdentity(state.input)
    )

    await act(async () => {
      await result.current.promoteConversationTitle({
        conversationId: "conversation-restored",
        currentTitle: "New chat",
        [titleKey]: "Analyze restored materials",
      })
    })

    expect(agentChatData.updateConversation).toHaveBeenCalledWith(
      "conversation-restored",
      { [titleKey]: "Analyze restored materials" }
    )
    expect(state.conversations[0]).toMatchObject({
      id: "conversation-restored",
      [titleKey]: "Analyze restored materials",
    })
  })

  it("does not promote an unknown restored title", async () => {
    const state = createIdentityState({
      conversationId: "conversation-unknown",
    })
    const { result } = renderHook(() =>
      useAgentChatConversationIdentity(state.input)
    )

    await act(async () => {
      await result.current.promoteConversationTitle({
        conversationId: "conversation-unknown",
        [titleKey]: "Analyze restored materials",
      })
    })

    expect(agentChatData.updateConversation).not.toHaveBeenCalled()
  })

  it("does not overwrite a non-fallback title", async () => {
    vi.mocked(agentChatData.createConversation).mockResolvedValue(
      createConversationDetail("conversation-manual", "Manual title")
    )
    const state = createIdentityState()
    const { result } = renderHook(() =>
      useAgentChatConversationIdentity(state.input)
    )

    await act(async () => {
      const conversationId = await result.current.ensureConversation({
        [titleKey]: "Manual title",
      })
      await result.current.promoteConversationTitle({
        conversationId,
        [titleKey]: "Analyze coffee products",
      })
    })

    expect(agentChatData.updateConversation).not.toHaveBeenCalled()
    expect(state.conversations[0]).toMatchObject({
      id: "conversation-manual",
      [titleKey]: "Manual title",
    })
  })
})

function createIdentityState(options: {
  conversationId?: string | null
  conversations?: AgentChatConversationItem[]
} = {}) {
  const conversationIdRef = { current: options.conversationId ?? null }
  const conversations: AgentChatConversationItem[] = [
    ...(options.conversations ?? []),
  ]
  const setConversationId: Dispatch<SetStateAction<string | null>> = vi.fn(
    (next) => {
      conversationIdRef.current =
        typeof next === "function" ? next(conversationIdRef.current) : next
    }
  )
  const setConversations = vi.fn(
    (
      updater:
        | AgentChatConversationItem[]
        | ((current: AgentChatConversationItem[]) => AgentChatConversationItem[])
    ) => {
      const next =
        typeof updater === "function" ? updater(conversations) : updater
      conversations.splice(0, conversations.length, ...next)
    }
  )

  return {
    conversationIdRef,
    conversations,
    input: {
      conversationIdRef,
      fallbackConversationTitle: "New chat",
      setConversationId,
      setConversations,
    },
  }
}

function createConversationDetail(
  id: string,
  title: string
): ConversationDetailResponse {
  return {
    conversation: createConversation(id, title),
    files: [],
    messages: [],
  }
}

function createConversation(id: string, title: string): ConversationResponse {
  return {
    id,
    [titleKey]: title,
    agentId: "web-agent",
    createdAt: "2026-07-16T00:00:00.000Z",
    modelName: null,
    source: "agent_chat",
    status: "active",
    updatedAt: "2026-07-16T00:00:00.000Z",
    userId: "user-one",
  }
}

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (reason?: unknown) => void
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise
    reject = rejectPromise
  })
  return { promise, reject, resolve }
}
