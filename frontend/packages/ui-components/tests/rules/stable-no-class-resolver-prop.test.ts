import { describe, expect, it } from "vitest"
import {
  findExportedPropMemberMatches,
  findStableTsxFiles,
  toFileExports,
} from "./ast-helpers"

describe("stable component class resolver props", () => {
  it("does not expose classResolver through exported stable props", () => {
    const matches = findExportedPropMemberMatches(
      findStableTsxFiles(),
      (propName) => propName === "classResolver"
    )

    expect(toFileExports(matches)).toEqual([])
  })

  it("does not expose classNameMode through exported stable props", () => {
    const matches = findExportedPropMemberMatches(
      findStableTsxFiles(),
      (propName) => propName === "classNameMode"
    )

    expect(toFileExports(matches)).toEqual([])
  })
})
