import { describe, expect, it } from "vitest"
import { parseLearningProgressView } from "./use-learning-progress-page"

describe("parseLearningProgressView", () => {
  it("supports direct links to each supplemental view", () => {
    expect(parseLearningProgressView("review")).toBe("review")
    expect(parseLearningProgressView("path")).toBe("path")
    expect(parseLearningProgressView("map")).toBe("map")
    expect(parseLearningProgressView("portfolio")).toBe("portfolio")
  })

  it("keeps the original overview as the safe default", () => {
    expect(parseLearningProgressView(null)).toBe("overview")
    expect(parseLearningProgressView("unknown")).toBe("overview")
  })
})
