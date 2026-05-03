import ts from "typescript"
import { describe, expect, it } from "vitest"
import {
  findStatementFindings,
  findUiCoreSourceFiles,
  toLocations,
} from "./ast-helpers"

describe("ui-core module exports", () => {
  it("does not use default exports in source files", () => {
    const findings = findStatementFindings(findUiCoreSourceFiles(), (node) => {
      if (ts.isExportAssignment(node)) {
        return true
      }

      if (!ts.canHaveModifiers(node) || !ts.getModifiers(node)?.length) {
        return false
      }

      return (
        ts
          .getModifiers(node)
          ?.some(
            (modifier) => modifier.kind === ts.SyntaxKind.DefaultKeyword
          ) ?? false
      )
    })

    expect(toLocations(findings)).toEqual([])
  })
})
