import { describe, expect, it } from "vitest"
import {
  findExportFindings,
  findPrimitiveIndexFiles,
  toLocations,
} from "./ast-helpers"

describe("ui-core primitive index protocols", () => {
  it("only re-exports local module files from primitive index files", () => {
    const findings = findExportFindings(findPrimitiveIndexFiles(), (node) => {
      const moduleSpecifier = node.moduleSpecifier?.getText().slice(1, -1)

      return !moduleSpecifier?.startsWith("./")
    })

    expect(toLocations(findings)).toEqual([])
  })
})
