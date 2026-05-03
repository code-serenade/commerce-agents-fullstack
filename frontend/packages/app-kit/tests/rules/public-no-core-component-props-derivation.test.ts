import { describe, expect, it } from "vitest"
import ts from "typescript"
import {
  findPublicSourceFiles,
  findTypeAliasFindings,
  toLocations,
} from "./ast-helpers"

describe("app public prop derivation", () => {
  it("does not derive public props from React.ComponentProps<typeof Core*>", () => {
    const findings = findTypeAliasFindings(findPublicSourceFiles(), (node) => {
      if (!node.name.text.endsWith("Props")) {
        return false
      }

      if (!ts.isTypeReferenceNode(node.type)) {
        return false
      }

      if (node.type.typeName.getText() !== "React.ComponentProps") {
        return false
      }

      const [firstArg] = node.type.typeArguments ?? []

      return Boolean(
        firstArg &&
          ts.isTypeQueryNode(firstArg) &&
          ts.isIdentifier(firstArg.exprName) &&
          firstArg.exprName.text.startsWith("Core")
      )
    })

    expect(toLocations(findings)).toEqual([])
  })
})
