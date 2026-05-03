import ts from "typescript"
import { describe, expect, it } from "vitest"
import {
  findExportFindings,
  findPrimitiveIndexFiles,
  toLocations,
} from "./ast-helpers"

describe("ui-core primitive index exports", () => {
  it("does not use export star in primitive index files", () => {
    const findings = findExportFindings(
      findPrimitiveIndexFiles(),
      (node) =>
        !node.exportClause ||
        (node.isTypeOnly === false && ts.isNamespaceExport(node.exportClause))
    )

    expect(toLocations(findings)).toEqual([])
  })
})
