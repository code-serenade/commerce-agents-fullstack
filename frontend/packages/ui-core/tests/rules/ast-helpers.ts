import { readdirSync, readFileSync, statSync } from "node:fs"
import path from "node:path"
import ts from "typescript"

const packageRoot = path.resolve(import.meta.dirname, "../..")
const srcRoot = path.join(packageRoot, "src")
const primitivesRoot = path.join(srcRoot, "primitives")

export interface RuleFinding {
  file: string
  line: number
  text: string
}

export function findUiCoreSourceFiles(): string[] {
  return walk(srcRoot)
    .filter((file) => /\.(ts|tsx)$/.test(file))
    .sort()
}

export function findPrimitiveDirectories(): string[] {
  return readdirSync(primitivesRoot)
    .map((name) => path.join(primitivesRoot, name))
    .filter((fullPath) => statSync(fullPath).isDirectory())
    .sort()
}

export function findPrimitiveIndexFiles(): string[] {
  return findPrimitiveDirectories().map((dir) => path.join(dir, "index.ts"))
}

export function getPrimitiveBaseName(dir: string): string {
  return path.basename(dir)
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

export function findStatementFindings(
  files: string[],
  matcher: (node: ts.Statement) => boolean
): RuleFinding[] {
  return files.flatMap((file) => {
    const { sourceFile } = readSourceFile(file)

    return sourceFile.statements.flatMap((statement) => {
      if (!matcher(statement)) {
        return []
      }

      return [toFinding(file, sourceFile, statement)]
    })
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
  const { line } = sourceFile.getLineAndCharacterOfPosition(
    node.getStart(sourceFile)
  )

  return {
    file: relativeFile,
    line: line + 1,
    text: node.getText(sourceFile),
  }
}

function walk(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const fullPath = path.join(dir, entry.name)
    return entry.isDirectory() ? walk(fullPath) : [fullPath]
  })
}
