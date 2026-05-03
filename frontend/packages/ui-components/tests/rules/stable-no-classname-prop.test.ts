import { describe, expect, it } from "vitest"
import {
  findExportedPropMemberMatches,
  findStableTsxFiles,
  toFileExports,
} from "./ast-helpers"

describe("stable component className props", () => {
  it("does not expose className through exported stable props", () => {
    const matches = findExportedPropMemberMatches(
      findStableTsxFiles(),
      (propName) => propName === "className"
    )

    expect(toFileExports(matches)).toEqual([])
  })
})
