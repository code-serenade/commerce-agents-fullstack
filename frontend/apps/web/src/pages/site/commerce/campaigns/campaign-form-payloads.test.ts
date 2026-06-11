import { describe, expect, it } from "vitest"
import {
  campaignFormDraftPayload,
  campaignFormDraftStreamPayload,
} from "./campaign-form-payloads"

const descriptionKey = "description"
const nameKey = "name"

describe("campaignFormDraftPayload", () => {
  it("keeps template smart fill as the default request mode", () => {
    const payload = campaignFormDraftPayload({
      budget: "3000",
      campaignType: "promotion",
      [descriptionKey]: "Plan a training campaign for campus live commerce.",
      endDate: "2026-06-30",
      [nameKey]: "Campus launch",
      productId: "",
      startDate: "2026-06-24",
    })

    expect(payload).not.toHaveProperty("generationMode")
    expect(payload).toMatchObject({
      budget: 3000,
      goal: "Campus launch",
      prompt: "Plan a training campaign for campus live commerce.",
    })
  })

  it("adds the explicit agent llm mode only when requested", () => {
    expect(
      campaignFormDraftPayload(
        {
          [descriptionKey]: "Fill this campaign through the system agent.",
          [nameKey]: "Agent campaign",
        },
        "agent_llm"
      )
    ).toMatchObject({
      generationMode: "agent_llm",
      goal: "Agent campaign",
      prompt: "Fill this campaign through the system agent.",
    })
  })

  it("builds the SSE stream payload with explicit agent mode", () => {
    expect(
      campaignFormDraftStreamPayload({
        [descriptionKey]: "Stream this campaign through the system agent.",
        [nameKey]: "Stream campaign",
      })
    ).toMatchObject({
      campaign: {
        generationMode: "agent_llm",
        goal: "Stream campaign",
        prompt: "Stream this campaign through the system agent.",
      },
      formType: "campaign",
    })
  })
})
