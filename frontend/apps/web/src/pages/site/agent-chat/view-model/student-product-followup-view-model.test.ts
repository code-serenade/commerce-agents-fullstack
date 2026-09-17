import { describe, expect, it } from "vitest"
import { getContextualProductFollowupPrompts } from "./student-product-followup-view-model"
import { textFromCodes } from "./student-next-actions-view-model-test-utils"

const t = ((_key: string, params?: Record<string, unknown>) =>
  params ? JSON.stringify(params) : "fallback") as never

describe("student product follow-up view model", () => {
  it("does not treat conversational first-person wording as an audience", () => {
    const prompts = getContextualProductFollowupPrompts(
      t,
      textFromCodes(
        21654,
        21857,
        32,
        36866,
        21512,
        25105,
        39,
        30340,
        38382,
        39064
      )
    ).join("\n")

    expect(prompts).toContain('"audience":"fallback"')
    expect(prompts).not.toContain(textFromCodes(25105, 39))
  })
})
