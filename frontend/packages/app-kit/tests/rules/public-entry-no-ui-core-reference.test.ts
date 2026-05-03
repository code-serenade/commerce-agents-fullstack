import { describe, expect, it } from "vitest"
import {
  findExportFindings,
  findImportFindings,
  findPublicEntryFiles,
  toLocations,
} from "./ast-helpers"

describe("app public entry primitive boundaries", () => {
  it("does not reference ui-core from public entry files", () => {
    const files = findPublicEntryFiles()
    const findings = [
      ...findImportFindings(files, (node) =>
        node.moduleSpecifier.getText().slice(1, -1).startsWith("@workspace/ui-core")
      ),
      ...findExportFindings(files, (node) =>
        node.moduleSpecifier?.getText().slice(1, -1).startsWith("@workspace/ui-core") ?? false
      ),
    ]

    expect(toLocations(findings)).toEqual([])
  })
})
