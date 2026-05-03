import { describe, expect, it } from "vitest"
import ts from "typescript"
import {
  findStableTsxFiles,
  findTypeAliasFindings,
  toFiles,
} from "./ast-helpers"

describe("stable component prop aliases", () => {
  it("does not expose Core*Props aliases from stable components", () => {
    const findings = findTypeAliasFindings(findStableTsxFiles(), (node) => {
      if (!node.name.text.endsWith("Props")) {
        return false
      }

      if (
        !ts.isTypeReferenceNode(node.type) ||
        !ts.isIdentifier(node.type.typeName)
      ) {
        return false
      }

      const typeName = node.type.typeName.text
      return typeName.startsWith("Core") && typeName.endsWith("Props")
    })

    expect(toFiles(findings)).toEqual([])
  })
})
