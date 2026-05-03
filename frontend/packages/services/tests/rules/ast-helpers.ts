import { existsSync, readFileSync } from "node:fs"
import path from "node:path"
import ts from "typescript"

const packageRoot = path.resolve(import.meta.dirname, "../..")

export interface RuleFinding {
  file: string
  line: number
  text: string
}

export function readPackageJson() {
  return JSON.parse(readFileSync(path.join(packageRoot, "package.json"), "utf8")) as {
    exports?: Record<string, string>
  }
}

export function resolveFromPackageRoot(relativePath: string) {
  return path.resolve(packageRoot, relativePath)
}

export function fileExists(relativePath: string) {
  return existsSync(resolveFromPackageRoot(relativePath))
}

export function findPublicEntryFiles(): string[] {
  const exportsMap = readPackageJson().exports ?? {}

  return Object.values(exportsMap)
    .filter((value): value is string => typeof value === "string")
    .map((value) => resolveFromPackageRoot(value))
    .sort()
}

export function readSourceFile(file: string) {
  const sourceText = readFileSync(file, "utf8")
  const sourceFile = ts.createSourceFile(
    file,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    file.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS
  )

  return { sourceFile, sourceText }
}

export function findImportFindings(
  files: string[],
  matcher: (node: ts.ImportDeclaration) => boolean
): RuleFinding[] {
  return files.flatMap((file) => {
    const { sourceFile } = readSourceFile(file)

    return sourceFile.statements.flatMap((statement) => {
      if (!ts.isImportDeclaration(statement) || !matcher(statement)) {
        return []
      }

      return [toFinding(file, sourceFile, statement)]
    })
  })
}

export function findExportFindings(
  files: string[],
  matcher: (node: ts.ExportDeclaration) => boolean
): RuleFinding[] {
  return files.flatMap((file) => {
    const { sourceFile } = readSourceFile(file)

    return sourceFile.statements.flatMap((statement) => {
      if (!ts.isExportDeclaration(statement) || !matcher(statement)) {
        return []
      }

      return [toFinding(file, sourceFile, statement)]
    })
  })
}

export function findJsxFindings(files: string[]): RuleFinding[] {
  return files.flatMap((file) => {
    const { sourceFile } = readSourceFile(file)
    const findings: RuleFinding[] = []

    visit(sourceFile, (node) => {
      if (
        ts.isJsxElement(node) ||
        ts.isJsxSelfClosingElement(node) ||
        ts.isJsxFragment(node)
      ) {
        findings.push(toFinding(file, sourceFile, node))
      }
    })

    return findings
  })
}

export function toLocations(findings: RuleFinding[]): string[] {
  return findings.map((finding) => `${finding.file}:${finding.line}`).sort()
}

function toFinding(
  file: string,
  sourceFile: ts.SourceFile,
  node: ts.Node
): RuleFinding {
  const relativeFile = path.relative(packageRoot, file)
  const { line } = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile))

  return {
    file: relativeFile,
    line: line + 1,
    text: node.getText(sourceFile),
  }
}

function visit(node: ts.Node, visitor: (node: ts.Node) => void) {
  visitor(node)
  node.forEachChild((child) => visit(child, visitor))
}
