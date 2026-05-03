import ts from "typescript"
import { describe, expect, it } from "vitest"
import { findJsxFindings, findStableTsxFiles, toFiles } from "./ast-helpers"

describe("stable component passthroughs", () => {
  it("does not directly passthrough props into Core* components", () => {
    const findings = findJsxFindings(findStableTsxFiles(), (node) => {
      const tagName = node.tagName.getText()

      if (!tagName.startsWith("Core")) {
        return false
      }

      return node.attributes.properties.some(
        (attribute) =>
          ts.isJsxSpreadAttribute(attribute) &&
          (attribute.expression.getText() === "props" ||
            attribute.expression.getText() === "rest")
      )
    })

    expect(toFiles(findings)).toEqual([])
  })
})
