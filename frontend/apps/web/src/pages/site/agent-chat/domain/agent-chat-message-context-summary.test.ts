import { describe, expect, it } from "vitest"
import {
  createMessageContextSummary,
  readMessageContextSummary,
} from "./agent-chat-message-context-summary"

describe("agent chat message context summary", () => {
  it("keeps selected product, campaign, and source names for the optimistic user message", () => {
    const productNameKey = "name"
    const sourceTitleKey = "title"
    const summary = createMessageContextSummary({
      campaignIds: ["campaign-a"],
      campaigns: [
        {
          budget: 3000,
          campaignType: "launch",
          id: "campaign-a",
          name: "Campaign A",
          status: "active",
        },
      ],
      contextRefs: ["product:product-a", "conversation_file:file-a", "campaign:campaign-a"],
      productIds: ["product-a"],
      products: [
        {
          brand: "Brand A",
          category: "Coffee",
          id: "product-a",
          [productNameKey]: "Product A",
          platform: "douyin",
          status: "active",
        },
      ],
      sources: [
        {
          readiness: "readable",
          ref: "conversation_file:file-a",
          [sourceTitleKey]: "brief.md",
        },
      ],
      strategyId: "collab",
    })

    expect(summary?.productIds).toEqual(["product-a"])
    expect(summary?.campaignIds).toEqual(["campaign-a"])
    expect(summary?.campaigns[0]?.id).toBe("campaign-a")
    expect(summary?.campaigns[0]?.label).toBe("Campaign A")
    expect(summary?.campaigns[0]?.secondary).toBe("active / launch / budget 3000")
    expect(summary?.products[0]?.id).toBe("product-a")
    expect(summary?.products[0]?.label).toBe("Product A")
    expect(summary?.products[0]?.secondary).toBe(
      "Brand A / douyin / Coffee / active"
    )
    expect(summary?.sourceRefs).toEqual(["conversation_file:file-a"])
    expect(summary?.sources[0]?.id).toBe("conversation_file:file-a")
    expect(summary?.sources[0]?.label).toBe("brief.md")
    expect(summary?.sources[0]?.readiness).toBe("readable")
    expect(summary?.strategyId).toBe("execution")
  })

  it("restores selected source readiness from persisted message context", () => {
    const sourceTitleKey = "title"
    const readableFileTitle = "readable.pdf"
    const pendingFileTitle = "pending.docx"
    const summary = readMessageContextSummary({
      context: {
        campaignIds: [],
        contextRefs: ["conversation_file:file-readable", "conversation_file:file-pending"],
        productIds: [],
        sources: [
          {
            readiness: "readable",
            ref: "conversation_file:file-readable",
            [sourceTitleKey]: readableFileTitle,
          },
          {
            readiness: "pending",
            ref: "conversation_file:file-pending",
            [sourceTitleKey]: pendingFileTitle,
          },
        ],
      },
      strategy: {
        strategyId: "execution",
      },
    })

    expect(summary?.sources).toEqual([
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
