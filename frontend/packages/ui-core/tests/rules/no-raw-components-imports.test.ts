import { describe, expect, it } from "vitest"
import {
  findImportFindings,
  findUiCoreSourceFiles,
  toLocations,
} from "./ast-helpers"

describe("ui-core raw component boundaries", () => {
  it("does not import from src/components raw generation files", () => {
    const findings = findImportFindings(findUiCoreSourceFiles(), (node) => {
      const moduleSpecifier = node.moduleSpecifier.getText().slice(1, -1)

      return (
        moduleSpecifier.includes("/components/") ||
        moduleSpecifier.startsWith("../components") ||
        moduleSpecifier.startsWith("../../components") ||
        moduleSpecifier.startsWith("./components")
      )
    })

    expect(toLocations(findings)).toEqual([])
  })
})
