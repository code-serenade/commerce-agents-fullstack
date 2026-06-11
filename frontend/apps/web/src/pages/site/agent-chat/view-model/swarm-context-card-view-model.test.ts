import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getContextCard } from "./swarm-context-card-view-model"

const packTitle = "Workspace pack"
const packMeta = "1/3 readable"
const packReadyDescription = "ready"
const warningLine = "warning unsupported pdf"
const omittedLine = "omitted file-b"
const itemTitle = ["SKU", "sheet"].join(" ")
const readableLine = "readable 1"

const t = ((key: string, params?: Record<string, unknown>) => {
  if (key === "agentChat.swarm.cards.workspaceContext.packTitle") return packTitle
  if (key === "agentChat.swarm.cards.workspaceContext.packMeta") {
    return `${params?.readable}/${params?.total} readable`
  }
  if (key === "agentChat.swarm.cards.workspaceContext.packReadyDescription") {
    return packReadyDescription
  }
  if (key === "agentChat.swarm.cards.workspaceContext.packPartialDescription") {
    return "partial"
  }
  if (key === "agentChat.swarm.cards.workspaceContext.packItemLine") {
    return `${params?.kind} ${params?.title}`
  }
  if (key === "agentChat.swarm.cards.workspaceContext.readableLine") {
    return `readable ${params?.count}`
  }
  if (key === "agentChat.swarm.cards.workspaceContext.compactLine") {
    return `sources ${params?.total}`
  }
  if (key === "agentChat.swarm.cards.workspaceContext.warningLine") {
    return `warning ${params?.warning}`
  }
  if (key === "agentChat.swarm.cards.workspaceContext.omittedLine") {
    return `omitted ${params?.ref}`
  }
  if (key === "agentChat.swarm.cards.ragContext.title") return "RAG context"
  if (key === "agentChat.swarm.cards.ragContext.meta") {
    return `${params?.sources}/${params?.knowledge} rag`
  }
  if (key === "agentChat.swarm.cards.ragContext.description") return "rag ready"
  if (key === "agentChat.swarm.cards.ragContext.degradedDescription") {
    return "rag degraded"
  }
  if (key === "agentChat.swarm.cards.ragContext.queryLine") {
    return `query ${params?.terms}`
  }
  if (key === "agentChat.swarm.cards.ragContext.fallbackLine") {
    return `fallback ${params?.reason}`
  }
  if (key === "agentChat.swarm.cards.contextLoaded.title") return "Product context"
  if (key === "agentChat.swarm.cards.contextLoaded.meta") {
    return `${params?.products}/${params?.knowledge} context`
  }
  if (key === "agentChat.swarm.cards.contextLoaded.description") return "context ready"
  if (key === "agentChat.swarm.cards.contextLoaded.productLine") {
    return `product ${params?.id}`
  }
  return key
}) as never

describe("getContextCard", () => {
  it("shows omitted material refs from workspace context pack events", () => {
    const hiddenPackSummary = "hidden workspace pack summary"
    const hiddenPackText = "hidden workspace pack text"
    const events: AgentRuntimeEvent[] = [
      {
        type: "workspace_context_pack_loaded",
        items: [
          {
            content_kind: "spreadsheet",
            id: "file-a",
            text: hiddenPackText,
            title: itemTitle,
          },
          { id: "file-b" },
          { id: "file-c" },
        ],
        omitted: ["file-b", "file-c"],
        readiness: "partial",
        summary: hiddenPackSummary,
        warnings: ["unsupported pdf"],
      },
    ]

    const card = getContextCard(events, t)
    expect(card).toMatchObject({
      description: packReadyDescription,
      lines: [readableLine, warningLine, "omitted file-b", "omitted file-c"],
      meta: packMeta,
      title: packTitle,
    })
    const serialized = JSON.stringify(card)
    expect(serialized).not.toContain(hiddenPackSummary)
    expect(serialized).not.toContain(hiddenPackText)
    expect(serialized).not.toContain(itemTitle)
  })

  it("keeps omitted refs visible when the material list is short", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "workspace_context_pack_loaded",
        items: [
          {
            content_kind: "spreadsheet",
            id: "file-a",
            text: "sku data",
            title: itemTitle,
          },
        ],
        omitted: ["file-b"],
        readiness: "partial",
        warnings: ["unsupported pdf"],
      },
    ]

    expect(getContextCard(events, t)?.lines).toEqual([
      readableLine,
      warningLine,
      omittedLine,
    ])
  })

  it("does not expose raw workspace summary or source previews in context cards", () => {
    const hiddenWorkspaceSummary = "hidden workspace summary"
    const hiddenSourcePreview = "hidden source preview"
    const hiddenSourceTitle = "hidden source title"
    const events: AgentRuntimeEvent[] = [
      {
        type: "workspace_context_loaded",
        attachmentSources: [
          {
            id: "file-a",
            preview: hiddenSourcePreview,
            refKey: "conversation_file:file-a",
            sourceType: "conversation_file",
            status: "ready",
            title: hiddenSourceTitle,
          },
        ],
        counts: {
          conversationFiles: 1,
        },
        readiness: "ready",
        summary: hiddenWorkspaceSummary,
        warnings: ["unsupported pdf"],
      },
    ]

    const card = getContextCard(events, t)
    expect(card?.lines).toEqual(["sources 1", warningLine])
    const serialized = JSON.stringify(card)
    expect(serialized).not.toContain(hiddenWorkspaceSummary)
    expect(serialized).not.toContain(hiddenSourcePreview)
    expect(serialized).not.toContain(hiddenSourceTitle)
  })

  it("uses safe workspace source count when source bodies are omitted", () => {
    const events: AgentRuntimeEvent[] = [
      {
        type: "workspace_context_loaded",
        counts: {
          campaigns: 1,
          conversationFiles: 1,
        },
        readiness: "ready",
        sourceCount: 5,
        warnings: [],
      },
    ]

    expect(getContextCard(events, t)?.lines).toEqual(["sources 5"])
  })

  it("does not expose raw rag summaries or source titles in context cards", () => {
    const hiddenSummary = "hidden rag summary with margins"
    const hiddenTitle = "hidden source title"
    const events: AgentRuntimeEvent[] = [
      {
        type: "rag_context_loaded",
        knowledgeCount: 2,
        queryTerms: ["coffee", "margin"],
        sourceCount: 3,
        sources: [
          {
            id: "knowledge-1",
            sourceType: "product_knowledge",
            title: hiddenTitle,
          },
        ],
        sourcesAvailable: true,
        summary: hiddenSummary,
        summaryAvailable: true,
      },
    ]

    const card = getContextCard(events, t)

    expect(card?.lines).toEqual(["query coffee, margin"])
    const serialized = JSON.stringify(card)
    expect(serialized).not.toContain(hiddenSummary)
    expect(serialized).not.toContain(hiddenTitle)
  })

  it("does not expose raw selected context summary in context cards", () => {
    const hiddenSummary = "hidden selected product facts"
    const titleKey = "title"
    const events: AgentRuntimeEvent[] = [
      {
        type: "context_loaded",
        knowledge_count: 2,
        product_count: 1,
        productIds: ["product-1"],
        source_count: 3,
        sourcesAvailable: true,
        sources_json: JSON.stringify([{ [titleKey]: "hidden context source" }]),
        summary: hiddenSummary,
        summaryAvailable: true,
      },
    ]

    const card = getContextCard(events, t)

    expect(card?.lines).toEqual(["product product-1"])
    const serialized = JSON.stringify(card)
    expect(serialized).not.toContain(hiddenSummary)
    expect(serialized).not.toContain("hidden context source")
  })
})
