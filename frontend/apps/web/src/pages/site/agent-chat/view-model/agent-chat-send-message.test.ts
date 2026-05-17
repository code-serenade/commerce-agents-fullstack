import { describe, expect, it, vi } from "vitest"
import { sendAgentChatMessage } from "./agent-chat-send-message"
import { agentChatData } from "../agent-chat-data"
import { getAgentChatStrategy } from "../domain/agent-chat-strategy"
import type { AgentChatMessage } from "../types"

vi.mock("../agent-chat-data", () => ({
  AGENT_ID: "web-agent",
  agentChatData: {
    addConversationMessage: vi.fn(),
    createConversation: vi.fn(),
    streamAgentSession: vi.fn(),
  },
}))

describe("sendAgentChatMessage", () => {
  it("sends empty product context when the user did not select products", async () => {
    const addConversationMessage = vi.mocked(agentChatData.addConversationMessage)
    const createConversation = vi.mocked(agentChatData.createConversation)
    const streamAgentSession = vi.mocked(agentChatData.streamAgentSession)
    const messages: AgentChatMessage[] = []
    const runtimeEvents: unknown[] = []

    createConversation.mockResolvedValue({
      conversation: {
        id: "conversation-a",
        ["title"]: "",
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
      content: "Analyze coffee category products.",
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
        modelName: "ollama-gpt-oss-20b",
        primaryRole: "ops",
        selectedRoles: ["ops", "data"],
      },
      selectedProducts: [],
      selectedSources: [],
      sessionIdRef: { current: "session-a" },
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
      setRuntimeEvents: (updater) => {
        if (typeof updater === "function") {
          runtimeEvents.splice(0, runtimeEvents.length, ...updater(runtimeEvents as never[]))
        }
      },
      strategy: getAgentChatStrategy("collab"),
      tError: "error",
    })

    expect(addConversationMessage).toHaveBeenCalledWith(
      "conversation-a",
      expect.objectContaining({
        contentJson: expect.objectContaining({
          context: {
            contextRefs: [],
            productIds: [],
          },
        }),
      })
    )
    expect(streamAgentSession).toHaveBeenCalledWith(
      expect.objectContaining({
        metadata: expect.objectContaining({
          context: {
            conversationId: "conversation-a",
            contextRefs: [],
            productIds: [],
          },
        }),
      }),
      expect.any(Object)
    )
    expect(messages[0]?.contextSummary?.productIds).toEqual([])
    expect(messages[0]?.contextSummary?.products).toEqual([])
    expect(messages[0]?.contextSummary?.sourceRefs).toEqual([])
    expect(messages[0]?.contextSummary?.sources).toEqual([])
  })

  it("cleans internal role quality failures from the live answer before saving", async () => {
    const addConversationMessage = vi.mocked(agentChatData.addConversationMessage)
    const createConversation = vi.mocked(agentChatData.createConversation)
    const streamAgentSession = vi.mocked(agentChatData.streamAgentSession)
    const messages: AgentChatMessage[] = []

    createConversation.mockResolvedValue({
      conversation: {
        id: "conversation-b",
        ["title"]: "",
        updatedAt: "2026-05-16T00:00:00.000Z",
      },
    } as Awaited<ReturnType<typeof agentChatData.createConversation>>)
    addConversationMessage.mockResolvedValue({} as Awaited<
      ReturnType<typeof agentChatData.addConversationMessage>
    >)
    streamAgentSession.mockImplementation(async (_body, handlers) => {
      handlers.onDelta(
        [
          textFromCodes([22810, 26234, 33021, 20307, 20219, 21153, 22270, 24050, 23436, 25104, 12290]),
          textFromCodes([26410, 23436, 25104, 33410, 28857, 65306]),
          `- ${textFromCodes([25968, 25454, 35786, 26029])}(data) failed: data role should provide at least two findings`,
        ].join("\n")
      )
      handlers.onDone()
    })

    await sendAgentChatMessage({
      content: `${textFromCodes([35831, 20998, 26512, 21654, 21857, 31867, 21830, 21697])}，${textFromCodes([19981, 24341, 29992, 20219, 20309, 39044, 35774, 21830, 21697])}。`,
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
        modelName: "ollama-gpt-oss-20b",
        primaryRole: "ops",
        selectedRoles: ["ops", "data"],
      },
      selectedProducts: [],
      selectedSources: [],
      sessionIdRef: { current: "session-b" },
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
      strategy: getAgentChatStrategy("collab"),
      tError: "error",
    })

    const assistant = messages.find((message) => message.role === "assistant")
    expect(assistant?.content).toContain("boundary notice")
    expect(assistant?.content).not.toContain("data role should provide")
    expect(addConversationMessage).toHaveBeenLastCalledWith(
      "conversation-b",
      expect.objectContaining({
        contentText: expect.not.stringContaining("data role should provide"),
        role: "assistant",
      })
    )
  })

  it("keeps late workspace events on the finished answer without reopening global streaming", async () => {
    const addConversationMessage = vi.mocked(agentChatData.addConversationMessage)
    const createConversation = vi.mocked(agentChatData.createConversation)
    const streamAgentSession = vi.mocked(agentChatData.streamAgentSession)
    const messages: AgentChatMessage[] = []
    const globalRuntimeEvents: unknown[] = []
    const streamingStates: boolean[] = []

    createConversation.mockResolvedValue({
      conversation: {
        id: "conversation-c",
        ["title"]: "",
        updatedAt: "2026-05-16T00:00:00.000Z",
      },
    } as Awaited<ReturnType<typeof agentChatData.createConversation>>)
    addConversationMessage.mockResolvedValue({} as Awaited<
      ReturnType<typeof agentChatData.addConversationMessage>
    >)
    streamAgentSession.mockImplementation(async (_body, handlers) => {
      handlers.onDelta("final answer")
      handlers.onAgentEvent?.({ type: "finished", session_id: "session-c" })
      handlers.onDone()
      handlers.onAgentEvent?.({
        type: "workspace_run_completed",
        id: "run-c",
        status: "succeeded",
      })
    })

    await sendAgentChatMessage({
      content: `${textFromCodes([35831, 29992, 22810, 23703, 20301, 21327, 20316, 20998, 26512, 30452, 25773, 38388, 28431, 26007])}，${textFromCodes([19981, 24341, 29992, 20219, 20309, 39044, 35774, 21830, 21697])}。`,
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
        modelName: "ollama-gpt-oss-20b",
        primaryRole: "ops",
        selectedRoles: ["ops", "data", "creative"],
      },
      selectedProducts: [],
      selectedSources: [],
      sessionIdRef: { current: "session-c" },
      setConversationId: vi.fn(),
      setConversations: vi.fn(),
      setError: vi.fn(),
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
      setRuntimeEvents: (updater) => {
        if (typeof updater === "function") {
          globalRuntimeEvents.splice(
            0,
            globalRuntimeEvents.length,
            ...updater(globalRuntimeEvents as never[])
          )
        }
      },
      strategy: getAgentChatStrategy("collab"),
      tError: "error",
    })

    const assistant = messages.find((message) => message.role === "assistant")
    expect(streamingStates).toContain(false)
    expect(globalRuntimeEvents).toEqual([
      { type: "finished", session_id: "session-c" },
    ])
    expect(assistant?.runtimeEvents).toEqual([
      { type: "finished", session_id: "session-c" },
      { type: "workspace_run_completed", id: "run-c", status: "succeeded" },
    ])
  })
})

function textFromCodes(codes: number[]) {
  return String.fromCodePoint(...codes)
}
