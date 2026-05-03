import { describe, expect, it } from "vitest"
import ts from "typescript"
import {
  findPublicSourceFiles,
  findTypeAliasFindings,
  toLocations,
} from "./ast-helpers"

describe("app public prop aliases", () => {
  it("does not export Core*Props aliases from public component sources", () => {
    const findings = findTypeAliasFindings(findPublicSourceFiles(), (node) => {
      if (!node.name.text.endsWith("Props")) {
        return false
      }

      if (!ts.isTypeReferenceNode(node.type) || !ts.isIdentifier(node.type.typeName)) {
        return false
      }

      const typeName = node.type.typeName.text
      return typeName.startsWith("Core") && typeName.endsWith("Props")
    })

    expect(toLocations(findings)).toEqual([])
  })
})
