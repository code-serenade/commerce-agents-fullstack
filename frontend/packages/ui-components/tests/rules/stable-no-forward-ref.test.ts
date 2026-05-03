import ts from "typescript"
import { describe, expect, it } from "vitest"
import {
  findCallExpressionFindings,
  findStableTsxFiles,
  toLocations,
} from "./ast-helpers"

describe("stable component ref protocols", () => {
  it("does not use forwardRef inside stable components", () => {
    const findings = findCallExpressionFindings(
      findStableTsxFiles(),
      (node) => {
        if (ts.isIdentifier(node.expression)) {
          return node.expression.text === "forwardRef"
        }

        return false
      }
    )

    expect(toLocations(findings)).toEqual([])
  })
})
