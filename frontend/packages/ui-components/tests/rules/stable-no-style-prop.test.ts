import { describe, expect, it } from "vitest"
import {
  findExportedPropMemberMatches,
  findStableTsxFiles,
  toFileExports,
} from "./ast-helpers"

describe("stable component style props", () => {
  it("does not expose style through exported stable props", () => {
    const matches = findExportedPropMemberMatches(
      findStableTsxFiles(),
      (propName) => propName === "style"
    )

    expect(toFileExports(matches)).toEqual([])
  })
})
