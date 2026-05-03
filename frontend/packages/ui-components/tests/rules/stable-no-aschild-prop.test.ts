import { describe, expect, it } from "vitest"
import {
  findExportedPropMemberMatches,
  findStableTsxFiles,
  toFileExports,
} from "./ast-helpers"

describe("stable component asChild props", () => {
  it("does not expose asChild through exported stable props", () => {
    const matches = findExportedPropMemberMatches(
      findStableTsxFiles(),
      (propName) => propName === "asChild"
    )

    expect(toFileExports(matches)).toEqual([])
  })
})
