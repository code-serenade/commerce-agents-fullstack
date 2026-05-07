import path from "node:path"
import { describe, expect, it } from "vitest"
import ts from "typescript"
import {
  appName,
  findImportFindings,
  findPageDirectories,
  findSourceFiles,
  getImportSpecifier,
  readSourceFile,
  toAppRelativePath,
  toLocations,
} from "./ast-helpers"

const maxLinesByRole = {
  component: 260,
  pageHook: 220,
  default: 320,
} as const

const forbiddenComponentValueImports = new Set([
  "@tanstack/react-query",
  "@/api",
])

const oversizedGuidance =
  "Split oversized page modules into data/domain/view-model/components. Components should render prepared props; page hooks should only orchestrate."

function getPageSourceFiles() {
  return findPageDirectories().flatMap((page) => findSourceFiles(page.dir))
}

function getModuleRole(file: string) {
  const normalized = file.split(path.sep).join("/")
  const fileName = path.basename(file)

  if (normalized.includes("/components/") || normalized.includes("/dialogs/")) {
    return "component"
  }

  if (/^use-[a-z0-9-]+-page\.ts$/.test(fileName)) {
    return "pageHook"
  }

  return "default"
}

function getMaxLines(file: string) {
  return maxLinesByRole[getModuleRole(file)]
}

function isTypeOnlyImport(node: ts.ImportDeclaration) {
  return node.importClause?.isTypeOnly === true
}

function isComponentSourceFile(file: string) {
  const normalized = file.split(path.sep).join("/")

  return file.endsWith(".tsx") && normalized.includes("/components/")
}

function isComponentFileWithImplementationState(file: string) {
  const { sourceFile } = readSourceFile(file)
  let hasReducer = false
  let hasComplexHookCount = 0

  function visit(node: ts.Node) {
    if (
      ts.isCallExpression(node) &&
      ts.isIdentifier(node.expression) &&
      node.expression.text === "useReducer"
    ) {
      hasReducer = true
    }

    if (
      ts.isCallExpression(node) &&
      ts.isIdentifier(node.expression) &&
      (node.expression.text === "useMemo" || node.expression.text === "useEffect")
    ) {
      hasComplexHookCount += 1
    }

    ts.forEachChild(node, visit)
  }

  visit(sourceFile)

  return hasReducer || hasComplexHookCount > 3
}

describe(`${appName} page module boundaries`, () => {
  it("keeps page modules small enough to preserve reviewable boundaries", () => {
    const oversized = getPageSourceFiles()
      .map((file) => {
        const { sourceText } = readSourceFile(file)

        return {
          file,
          lines: sourceText.split(/\r?\n/).length,
          maxLines: getMaxLines(file),
        }
      })
      .filter((entry) => entry.lines > entry.maxLines)
      .map(
        (entry) =>
          `${toAppRelativePath(entry.file)}:${entry.lines} > ${entry.maxLines}. ${oversizedGuidance}`
      )
      .sort()

    expect(oversized).toEqual([])
  })

  it("keeps presentational page components away from API and query wiring", () => {
    const findings = findImportFindings(
      getPageSourceFiles().filter(isComponentSourceFile),
      (node) => {
        if (isTypeOnlyImport(node)) {
          return false
        }

        return forbiddenComponentValueImports.has(getImportSpecifier(node))
      }
    )

    expect(toLocations(findings)).toEqual([])
  })

  it("keeps complex reducer and heavy derived state out of page components", () => {
    const mixedComponents = getPageSourceFiles()
      .filter(isComponentSourceFile)
      .filter(isComponentFileWithImplementationState)
      .map(
        (file) =>
          `${toAppRelativePath(file)}. Move reducer/state derivation into use-<page>-page.ts, view-model/, or domain/logic.ts.`
      )
      .sort()

    expect(mixedComponents).toEqual([])
  })
})
