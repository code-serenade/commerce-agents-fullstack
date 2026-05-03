import { describe, expect, it } from "vitest"
import ts from "typescript"
import {
  appName,
  findImportFindings,
  findPageDirectories,
  getImportSpecifier,
  readSourceFile,
  toLocations,
} from "./ast-helpers"

const pageIndexFiles = findPageDirectories().map((page) => page.indexFile)
const maxPageIndexLines = 160
const forbiddenValueImports = new Set([
  "@/api",
  "@tanstack/react-query",
  "react-hook-form",
  "@hookform/resolvers/zod",
  "zod",
])

function isTypeOnlyImport(node: ts.ImportDeclaration) {
  return node.importClause?.isTypeOnly === true
}

describe(`${appName} page index boundaries`, () => {
  it("keeps page index files small enough to stay as assembly", () => {
    const oversized = pageIndexFiles
      .map((file) => {
        const { sourceText } = readSourceFile(file)
        return {
          file,
          lines: sourceText.split(/\r?\n/).length,
        }
      })
      .filter((entry) => entry.lines > maxPageIndexLines)
      .map((entry) => `${entry.file}:${entry.lines}`)

    expect(oversized).toEqual([])
  })

  it("keeps page indexes away from request and form implementation wiring", () => {
    const findings = findImportFindings(pageIndexFiles, (node) => {
      if (isTypeOnlyImport(node)) {
        return false
      }

      const specifier = getImportSpecifier(node)

      return forbiddenValueImports.has(specifier)
    })

    expect(toLocations(findings)).toEqual([])
  })
})
