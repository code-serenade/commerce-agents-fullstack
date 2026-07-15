import { describe, expect, it } from "vitest"
import {
  productPayload,
  productDraftPayload,
  productDraftStreamPayload,
} from "./product-form-payloads"

const descriptionKey = "description"
const nameKey = "name"

describe("productDraftPayload", () => {
  it("normalizes product prices to a non-negative two-decimal value", () => {
    expect(
      productPayload({
        currency: "CNY",
        name: "Price test",
        price: "-12.349",
      })
    ).toMatchObject({
      currency: "CNY",
      price: "12.34",
    })
  })

  it("keeps template smart fill as the default request mode", () => {
    const payload = productDraftPayload({
      brand: "Atlas",
      category: "Drinkware",
      currency: "CNY",
      [descriptionKey]: "Insulated cup for a campus livestream task.",
      [nameKey]: "Insulated cup",
      platform: "douyin",
      price: "69",
    })

    expect(payload).not.toHaveProperty("generationMode")
    expect(payload).toMatchObject({
      currency: "CNY",
      name: "Insulated cup",
      prompt: "Insulated cup for a campus livestream task.",
    })
  })

  it("adds the explicit agent llm mode only when requested", () => {
    expect(
      productDraftPayload(
        {
          [descriptionKey]: "Fill this product through the system agent.",
          [nameKey]: "Agent product",
        },
        "agent_llm"
      )
    ).toMatchObject({
      generationMode: "agent_llm",
      name: "Agent product",
      prompt: "Fill this product through the system agent.",
    })
  })

  it("builds the SSE stream payload with explicit agent mode", () => {
    expect(
      productDraftStreamPayload({
        [descriptionKey]: "Stream this product through the system agent.",
        [nameKey]: "Stream product",
      })
    ).toMatchObject({
      formType: "product",
      product: {
        generationMode: "agent_llm",
        name: "Stream product",
        prompt: "Stream this product through the system agent.",
      },
    })
  })
})
