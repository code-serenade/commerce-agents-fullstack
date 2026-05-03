import { describe, expect, it } from "vitest"
import { findJsxFindings, findPublicEntryFiles, toLocations } from "./ast-helpers"

describe("services public entry view boundaries", () => {
  it("does not contain JSX in public entry files", () => {
    const findings = findJsxFindings(findPublicEntryFiles())

    expect(toLocations(findings)).toEqual([])
  })
})
