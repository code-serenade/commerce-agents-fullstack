import ts from "typescript"
import { describe, expect, it } from "vitest"
import { findJsxFindings, findStableTsxFiles, toLocations } from "./ast-helpers"

describe("stable component trigger protocols", () => {
  it("does not use Trigger asChild inside stable components", () => {
    const findings = findJsxFindings(findStableTsxFiles(), (node) => {
      const tagName = node.tagName.getText()

      if (!tagName.endsWith("Trigger")) {
        return false
      }

      return node.attributes.properties.some(
        (attribute) =>
          ts.isJsxAttribute(attribute) && attribute.name.getText() === "asChild"
      )
    })

    expect(toLocations(findings)).toEqual([])
  })
})
