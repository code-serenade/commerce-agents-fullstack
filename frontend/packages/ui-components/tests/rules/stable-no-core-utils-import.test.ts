import { describe, expect, it } from "vitest"
import {
  findImportFindings,
  findStableTsxFiles,
  toLocations,
} from "./ast-helpers"

describe("stable component core utility imports", () => {
  it("does not import ui-core utility modules inside stable components", () => {
    const findings = findImportFindings(findStableTsxFiles(), (node) => {
      const moduleSpecifier = node.moduleSpecifier.getText().slice(1, -1)

      return (
        moduleSpecifier === "@workspace/ui-core/lib/utils" ||
        moduleSpecifier === "@workspace/ui-core/lib/utils.js"
      )
    })

    expect(toLocations(findings)).toEqual([])
  })
})
