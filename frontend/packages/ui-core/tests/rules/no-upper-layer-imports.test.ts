import { describe, expect, it } from "vitest"
import {
  findImportFindings,
  findUiCoreSourceFiles,
  toLocations,
} from "./ast-helpers"

describe("ui-core dependency boundaries", () => {
  it("does not import upper-layer workspace packages", () => {
    const findings = findImportFindings(findUiCoreSourceFiles(), (node) => {
      const moduleSpecifier = node.moduleSpecifier.getText().slice(1, -1)

      return (
        moduleSpecifier.startsWith("@workspace/ui-components") ||
        moduleSpecifier.startsWith("@workspace/app-kit")
      )
    })

    expect(toLocations(findings)).toEqual([])
  })
})
