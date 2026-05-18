import { describe, expect, it } from "vitest"
import { createMessageContextSummary } from "./agent-chat-message-context-summary"

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
    expect(summary?.strategyId).toBe("execution")
  })
})
