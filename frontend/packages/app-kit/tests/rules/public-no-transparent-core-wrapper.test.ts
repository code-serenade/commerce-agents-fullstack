import { describe, expect, it } from "vitest"
import ts from "typescript"
import {
  findPublicSourceFiles,
  findSourceFilesMatching,
  toFiles,
} from "./ast-helpers"

describe("app public wrapper boundaries", () => {
  it("does not expose transparent ui-core passthrough wrappers", () => {
    const matches = findSourceFilesMatching(findPublicSourceFiles(), (sourceFile) => {
      let importsCoreAlias = false
      let exportsCoreProps = false
      let forwardsProps = false

      const visit = (node: ts.Node) => {
        if (ts.isImportDeclaration(node)) {
          const moduleSpecifier = node.moduleSpecifier.getText().slice(1, -1)

          if (moduleSpecifier.startsWith("@workspace/ui-core/")) {
            const namedBindings = node.importClause?.namedBindings

            if (namedBindings && ts.isNamedImports(namedBindings)) {
              for (const element of namedBindings.elements) {
                if (element.name.text.startsWith("Core")) {
                  importsCoreAlias = true
                }
              }
            }
          }
        }

        if (ts.isTypeAliasDeclaration(node) && node.name.text.endsWith("Props")) {
          if (ts.isTypeReferenceNode(node.type) && ts.isIdentifier(node.type.typeName)) {
            const typeName = node.type.typeName.text

            if (typeName.startsWith("Core") && typeName.endsWith("Props")) {
              exportsCoreProps = true
            }
          }
        }

        if (ts.isJsxSelfClosingElement(node) || ts.isJsxOpeningElement(node)) {
          const tagName = node.tagName.getText()

          if (tagName.startsWith("Core")) {
            for (const attribute of node.attributes.properties) {
              if (
                ts.isJsxSpreadAttribute(attribute) &&
                attribute.expression.getText() === "props"
              ) {
                forwardsProps = true
              }
            }
          }
        }

        ts.forEachChild(node, visit)
      }

      visit(sourceFile)

      return importsCoreAlias && exportsCoreProps && forwardsProps
    })

    expect(toFiles(matches)).toEqual([])
  })
})
