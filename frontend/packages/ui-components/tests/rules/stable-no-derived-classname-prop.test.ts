import { describe, expect, it } from "vitest"
import {
  findExportedPropMemberMatches,
  findStableTsxFiles,
  toFileExports,
} from "./ast-helpers"

describe("stable component derived className props", () => {
  it("does not expose *ClassName props through exported stable props", () => {
    const matches = findExportedPropMemberMatches(
      findStableTsxFiles(),
      (propName) => propName.endsWith("ClassName")
    )

    expect(toFileExports(matches)).toEqual([])
  })
})
