import { existsSync, readFileSync, readdirSync, statSync } from "node:fs"
import path from "node:path"
import ts from "typescript"

export const packageRoot = process.env.APP_ROOT
  ? path.resolve(process.env.APP_ROOT)
  : process.cwd()
export const appName = path.basename(packageRoot)

const pageRootNames = ["auth", "protected", "public", "site"] as const
const sourceExtensions = [".ts", ".tsx"] as const

export interface RuleFinding {
  file: string
  line: number
  text: string
}

export interface PageDirectory {
  name: string
  dir: string
  dataFile: string | null
  indexFile: string
}

export function resolveAppPath(...segments: string[]) {
  return path.join(packageRoot, ...segments)
}

export function fileExists(file: string) {
  return existsSync(file)
}

export function isDirectory(file: string) {
  return fileExists(file) && statSync(file).isDirectory()
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

export function findNodeFindings(
  files: string[],
  matcher: (node: ts.Node, sourceFile: ts.SourceFile) => boolean
): RuleFinding[] {
  return files.flatMap((file) => {
    const { sourceFile } = readSourceFile(file)
    const findings: RuleFinding[] = []

    visit(sourceFile, (node) => {
      if (matcher(node, sourceFile)) {
        findings.push(toFinding(file, sourceFile, node))
      }
    })

    return findings
  })
}

export function getImportSpecifier(node: ts.ImportDeclaration) {
  return node.moduleSpecifier.getText().replace(/["']/g, "")
}

export function toLocations(findings: RuleFinding[]): string[] {
  return findings.map((finding) => `${finding.file}:${finding.line}`).sort()
}

export function toAppRelativePath(file: string) {
  return path.relative(packageRoot, file)
}

export function getExistingPageRoots() {
  return pageRootNames
    .map((name) => resolveAppPath("src", "pages", name))
    .filter(fileExists)
}

export function findSourceFiles(root: string): string[] {
  if (!fileExists(root)) {
    return []
  }

  if (!isDirectory(root)) {
    return sourceExtensions.some((extension) => root.endsWith(extension))
      ? [root]
      : []
  }

  const entries = readdirSync(root, { withFileTypes: true })

  return entries.flatMap((entry) => {
    const file = path.join(root, entry.name)

    if (entry.isDirectory()) {
      return findSourceFiles(file)
    }

    return sourceExtensions.some((extension) => file.endsWith(extension))
      ? [file]
      : []
  })
}

export function findAppSourceFiles() {
  return findSourceFiles(resolveAppPath("src"))
}

export function findPageDirectories(): PageDirectory[] {
  return getExistingPageRoots().flatMap((pageRoot) =>
    walkDirectories(pageRoot)
      .filter((dir) => fileExists(path.join(dir, "index.tsx")))
      .map((dir) => {
        const pageName = path.basename(dir)
        const dataFile = path.join(dir, `${pageName}-data.tsx`)

        return {
          name: toAppRelativePath(dir),
          dir,
          dataFile: fileExists(dataFile) ? dataFile : null,
          indexFile: path.join(dir, "index.tsx"),
        }
      })
  )
}

export function hasImport(file: string, specifier: string) {
  if (!fileExists(file)) {
    return false
  }

  const { sourceFile } = readSourceFile(file)

  return sourceFile.statements.some(
    (statement) =>
      ts.isImportDeclaration(statement) && getImportSpecifier(statement) === specifier
  )
}

function walkDirectories(root: string): string[] {
  const entries = readdirSync(root, { withFileTypes: true })
  const childDirs = entries
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(root, entry.name))

  return [root, ...childDirs.flatMap(walkDirectories)]
}

function toFinding(
  file: string,
  sourceFile: ts.SourceFile,
  node: ts.Node
): RuleFinding {
  const relativeFile = toAppRelativePath(file)
  const { line } = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile))

  return {
    file: relativeFile,
    line: line + 1,
    text: node.getText(sourceFile),
  }
}

function visit(node: ts.Node, callback: (node: ts.Node) => void) {
  callback(node)
  ts.forEachChild(node, (child) => visit(child, callback))
}
