import { describe, expect, it } from "vitest"
import {
  findExportFindings,
  findPublicEntryFiles,
  toLocations,
} from "./ast-helpers"

describe("app public entry protocols", () => {
  it("only re-exports local modules from public entry files", () => {
    const findings = findExportFindings(findPublicEntryFiles(), (node) => {
      const moduleSpecifier = node.moduleSpecifier?.getText().slice(1, -1)
      return Boolean(moduleSpecifier && !moduleSpecifier.startsWith("./"))
    })

    expect(toLocations(findings)).toEqual([])
  })
})
