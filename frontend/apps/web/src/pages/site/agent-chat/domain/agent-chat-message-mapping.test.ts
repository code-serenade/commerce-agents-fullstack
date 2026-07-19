import { describe, expect, it } from "vitest"
import type { ConversationDetailResponse } from "@/api"
import {
  createConversationTitle,
  isFallbackConversationTitle,
  mapConversationAttachments,
  mapConversationMessages,
  readConversationRuntimeEvents,
  readConversationRuntimeSelection,
  readConversationSelectedProductIds,
  readConversationSelectedSourceRefs,
  readConversationStrategyId,
} from "./agent-chat-message-mapping"
import { readMessageContextSummary } from "./agent-chat-message-context-summary"
import type { AgentChatMessage } from "../types"

const fallback: AgentChatMessage = {
  id: "welcome",
  role: "assistant",
  content: "welcome",
}

const copy = {
  boundaryNotice: "boundary notice",
  missingStructuredOutput: "missing structured output",
  streamTimedOut: "stream timed out",
}

describe("agent chat message mapping", () => {
  it("creates trimmed conversation titles from the first user message", () => {
    expect(createConversationTitle("  Analyze coffee products  ")).toBe(
      "Analyze coffee products"
    )
    expect(createConversationTitle("  ")).toBe("")
    expect(createConversationTitle("A".repeat(45))).toBe(
      `${"A".repeat(40)}...`
    )
  })

  it("recognizes empty and fallback conversation titles", () => {
    expect(isFallbackConversationTitle(null, "New chat")).toBe(true)
    expect(isFallbackConversationTitle("", "New chat")).toBe(true)
    expect(isFallbackConversationTitle("New chat", "New chat")).toBe(true)
    expect(isFallbackConversationTitle("Manual title", "New chat")).toBe(false)
  })

  it("restores persisted runtime events from assistant messages", () => {
    const detail = conversationDetail([
      {
        id: "m1",
        role: "user",
        contentText: "diagnose funnel",
        contentJson: null,
        sequence: 1,
      },
      {
        id: "m2",
        role: "assistant",
        contentText: "answer",
        contentJson: {
          runtimeEvents: [
            {
              type: "route_decided",
              route_mode: "task_graph",
              support_runtime_roles: ["data"],
            },
          ],
        },
        sequence: 2,
      },
    ])

    expect(mapConversationMessages(detail, fallback, copy)).toEqual([
      {
        id: "m1",
        role: "user",
        content: "diagnose funnel",
        runtimeEvents: [],
      },
      {
        id: "m2",
        role: "assistant",
        content: "answer",
        runtimeEvents: [
          {
            type: "route_decided",
            route_mode: "task_graph",
            support_runtime_roles: ["data"],
          },
        ],
      },
    ])
    expect(readConversationRuntimeEvents(detail)).toEqual([
      {
        type: "route_decided",
        route_mode: "task_graph",
        support_runtime_roles: ["data"],
      },
    ])
  })

  it("restores selected product ids from the latest persisted user context", () => {
    const detail = conversationDetail([
      {
        id: "m1",
        role: "user",
        contentText: "first question",
        contentJson: {
          context: {
            productIds: ["product-a"],
            contextRefs: ["product:product-a"],
          },
        },
        sequence: 1,
      },
      {
        id: "m2",
        role: "assistant",
        contentText: "answer",
        contentJson: null,
        sequence: 2,
      },
      {
        id: "m3",
        role: "user",
        contentText: "follow up",
        contentJson: {
          context: {
            productIds: ["product-b"],
            contextRefs: ["product:product-b"],
          },
        },
        sequence: 3,
      },
    ])

    expect(readConversationSelectedProductIds(detail)).toEqual(["product-b"])
  })

  it("restores selected source refs from the latest persisted user context", () => {
    const fileRef = "conversation_file:file-a"
    const detail = conversationDetail([
      {
        id: "m1",
        role: "user",
        contentText: "first question",
        contentJson: {
          context: {
            contextRefs: [fileRef],
          },
        },
        sequence: 1,
      },
      {
        id: "m2",
        role: "assistant",
        contentText: "answer",
        contentJson: null,
        sequence: 2,
      },
    ])

    expect(readConversationSelectedSourceRefs(detail)).toEqual([fileRef])
  })

  it("restores strategy and runtime selection from the latest persisted user context", () => {
    const detail = conversationDetail([
      {
        id: "m1",
        role: "user",
        contentText: "first question",
        contentJson: {
          strategy: {
            collaborationMode: "manual",
            learningLevel: "higher_vocational",
            primaryRole: "data",
            selectedRoles: ["data", "creative"],
            strategyId: "collab",
          },
        },
        sequence: 1,
      },
    ])

    expect(readConversationStrategyId(detail)).toBe("execution")
    expect(readConversationRuntimeSelection(detail)).toEqual({
      collaborationMode: "manual",
      learningLevel: "higher_vocational",
      primaryRole: "data",
      selectedRoles: ["data", "creative"],
    })
  })

  it("maps message-level context summaries from persisted user metadata", () => {
    const fileRef = "conversation_file:file-a"
    const contentJson = {
      context: {
        contextRefs: ["product:product-a", fileRef],
        productIds: ["product-a"],
      },
      strategy: {
        collaborationMode: "manual",
        learningLevel: "higher_vocational",
        primaryRole: "data",
        selectedRoles: ["data", "creative"],
        strategyId: "collab",
      },
    }
    const detail = conversationDetail([
      {
        id: "m1",
        role: "user",
        contentText: "diagnose conversion",
        contentJson,
        sequence: 1,
      },
    ])
    const expectedSummary = {
      campaignIds: [],
      campaigns: [],
      collaborationMode: "manual",
      learningLevel: "higher_vocational",
      primaryRole: "data",
      productIds: ["product-a"],
      products: [{ id: "product-a" }],
      selectedRoles: ["data", "creative"],
      sourceRefs: [fileRef],
      sources: [{ id: fileRef }],
      strategyId: "execution",
    }

    expect(readMessageContextSummary(contentJson)).toEqual(expectedSummary)
    expect(mapConversationMessages(detail, fallback, copy)[0]).toEqual({
      id: "m1",
      role: "user",
      content: "diagnose conversion",
      contextSummary: expectedSummary,
      runtimeEvents: [],
    })
  })

  it("maps persisted conversation files into composer attachments", () => {
    const detail = conversationDetail([])
    detail.files = [
      {
        bucket: "agent-files",
        conversationId: "c1",
        createdAt: "2026-05-13T00:00:00.000Z",
        fileName: "brief.md",
        fileSize: 120,
        id: "file-1",
        messageId: null,
        metadata: {},
        mimeType: "text/markdown",
        objectKey: "conversation/file-1.md",
        sha256: null,
        status: "active",
      },
    ]

    expect(mapConversationAttachments(detail)).toEqual([
      {
        bucket: "agent-files",
        fileId: "file-1",
        id: "file-1",
        imageDataUrl: undefined,
        imageDetail: undefined,
        kind: "file",
        name: "brief.md",
        objectKey: "conversation/file-1.md",
        readable: true,
        status: "uploaded",
        unsupportedReason: undefined,
      },
    ])
  })

})

function conversationDetail(
  messages: Array<{
    id: string
    role: "user" | "assistant"
    contentText: string
    contentJson: unknown
    sequence: number
  }>
): ConversationDetailResponse {
  return {
    conversation: {
      agentId: "web-agent", createdAt: "2026-05-13T00:00:00.000Z", id: "c1", modelName: null, source: "agent_chat", status: "active", title: null,
      updatedAt: "2026-05-13T00:00:00.000Z", userId: "u1",
    },
    files: [],
    messages: messages.map((message) => ({
      conversationId: "c1",
      createdAt: "2026-05-13T00:00:00.000Z",
      id: message.id,
      role: message.role,
      status: "active",
      updatedAt: "2026-05-13T00:00:00.000Z",
      userId: "u1",
      contentJson: message.contentJson,
      contentText: message.contentText,
      sequence: message.sequence,
    })),
  }
}
