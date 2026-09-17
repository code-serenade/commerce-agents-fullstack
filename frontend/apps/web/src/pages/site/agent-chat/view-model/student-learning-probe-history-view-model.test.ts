import { describe, expect, it } from "vitest"
import {
  createProbe,
  createResult,
} from "./use-student-learning-probe-test-utils"
import { mergeCompletedLearningProbes } from "./student-learning-probe-history-view-model"

describe("student learning probe history view model", () => {
  it("keeps server order, lets local results win, and removes the visible probe", () => {
    const first = {
      probe: createProbe({ id: "first" }),
      result: createResult({ id: "first" }),
    }
    const visible = {
      probe: createProbe({ id: "visible" }),
      result: createResult({ id: "visible" }),
    }
    const refreshedFirst = {
      probe: createProbe({ id: "first" }),
      result: createResult({ id: "first", outcome: "incorrect" }),
    }

    const merged = mergeCompletedLearningProbes(
      [first, visible],
      [refreshedFirst],
      visible.probe.id
    )

    expect(merged).toEqual([refreshedFirst])
  })
})
