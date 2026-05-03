import { describe, expect, it } from "vitest"
import {
  findExportFindings,
  findPublicSourceFiles,
  toLocations,
} from "./ast-helpers"

describe("app public re-export boundaries", () => {
  it("does not directly re-export ui-core modules from public sources", () => {
    const findings = findExportFindings(findPublicSourceFiles(), (node) =>
      node.moduleSpecifier?.getText().slice(1, -1).startsWith("@workspace/ui-core/") ?? false
    )

    expect(toLocations(findings)).toEqual([])
  })
})
