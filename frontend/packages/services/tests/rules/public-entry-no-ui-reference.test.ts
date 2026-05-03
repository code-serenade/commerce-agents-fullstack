import { describe, expect, it } from "vitest"
import {
  findExportFindings,
  findImportFindings,
  findPublicEntryFiles,
  toLocations,
} from "./ast-helpers"

describe("services public entry boundaries", () => {
  it("does not reference ui packages or app code from public entry files", () => {
    const files = findPublicEntryFiles()
    const findings = [
      ...findImportFindings(files, (node) => {
        const moduleSpecifier = node.moduleSpecifier.getText().slice(1, -1)

        return (
          moduleSpecifier.startsWith("@workspace/ui-") ||
          moduleSpecifier.startsWith("@/") ||
          moduleSpecifier.startsWith("apps/")
        )
      }),
      ...findExportFindings(files, (node) => {
        const moduleSpecifier = node.moduleSpecifier?.getText().slice(1, -1) ?? ""

        return (
          moduleSpecifier.startsWith("@workspace/ui-") ||
          moduleSpecifier.startsWith("@/") ||
          moduleSpecifier.startsWith("apps/")
        )
      }),
    ]

    expect(toLocations(findings)).toEqual([])
  })
})
