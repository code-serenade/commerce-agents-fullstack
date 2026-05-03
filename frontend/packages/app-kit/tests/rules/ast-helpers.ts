import { readFileSync, readdirSync } from "node:fs"
import path from "node:path"
import ts from "typescript"

const packageRoot = path.resolve(import.meta.dirname, "../..")

export interface RuleFinding {
  file: string
  line: number
  text: string
}

export interface ExportedPropMemberMatch {
  file: string
  exportName: string
  propName: string
}

export interface SourceFileMatch {
  file: string
  text: string
}

export function readPackageJson() {
  return JSON.parse(readFileSync(path.join(packageRoot, "package.json"), "utf8")) as {
    exports?: Record<string, string>
  }
}

export function findPublicEntryFiles(): string[] {
  const exportsMap = readPackageJson().exports ?? {}

  return Object.values(exportsMap)
    .filter((value): value is string => typeof value === "string")
    .map((value) => path.resolve(packageRoot, value))
    .sort()
}

export function findPublicSourceFiles(): string[] {
  const files = new Set<string>()

  for (const entryFile of findPublicEntryFiles()) {
    files.add(entryFile)

    if (
      entryFile.endsWith("/index.ts") &&
      (entryFile.includes("/src/components/") ||
        entryFile.includes("/src/pages/"))
    ) {
      for (const file of walk(path.dirname(entryFile))) {
        if (isSourceFile(file)) {
          files.add(file)
        }
      }
    }
  }

  return [...files].sort()
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

export function findTypeAliasFindings(
  files: string[],
  matcher: (node: ts.TypeAliasDeclaration) => boolean
): RuleFinding[] {
  return files.flatMap((file) => {
    const { sourceFile } = readSourceFile(file)

    return sourceFile.statements.flatMap((statement) => {
      if (!ts.isTypeAliasDeclaration(statement) || !matcher(statement)) {
        return []
      }

      return [toFinding(file, sourceFile, statement)]
    })
  })
}

export function findJsxFindings(
  files: string[],
  matcher: (
    node: ts.JsxOpeningElement | ts.JsxSelfClosingElement
  ) => boolean
): RuleFinding[] {
  return files.flatMap((file) => {
    const { sourceFile } = readSourceFile(file)
    const findings: RuleFinding[] = []

    visit(sourceFile, (node) => {
      if (
        (ts.isJsxOpeningElement(node) || ts.isJsxSelfClosingElement(node)) &&
        matcher(node)
      ) {
        findings.push(toFinding(file, sourceFile, node))
      }
    })

    return findings
  })
}

export function findSourceFilesMatching(
  files: string[],
  matcher: (sourceFile: ts.SourceFile, sourceText: string) => boolean
): SourceFileMatch[] {
  return files.flatMap((file) => {
    const { sourceFile, sourceText } = readSourceFile(file)

    return matcher(sourceFile, sourceText)
      ? [
          {
            file: path.relative(packageRoot, file),
            text: sourceText,
          },
        ]
      : []
  })
}

export function findExportedPropMemberMatches(
  files: string[],
  matcher: (propName: string, typeText: string) => boolean
): ExportedPropMemberMatch[] {
  return files.flatMap((file) => {
    const { sourceFile } = readSourceFile(file)

    return sourceFile.statements.flatMap((statement) => {
      if (ts.isInterfaceDeclaration(statement) && isExported(statement)) {
        if (!statement.name.text.endsWith("Props")) {
          return []
        }

        return collectInterfacePropertyEntries(statement)
          .filter(({ propName, typeText }) => matcher(propName, typeText))
          .map(({ propName }) => ({
            file: path.relative(packageRoot, file),
            exportName: statement.name.text,
            propName,
          }))
      }

      if (ts.isTypeAliasDeclaration(statement) && isExported(statement)) {
        if (!statement.name.text.endsWith("Props")) {
          return []
        }

        return collectTypePropertyEntries(statement.type)
          .filter(({ propName, typeText }) => matcher(propName, typeText))
          .map(({ propName }) => ({
            file: path.relative(packageRoot, file),
            exportName: statement.name.text,
            propName,
          }))
      }

      return []
    })
  })
}

export function toLocations(findings: RuleFinding[]): string[] {
  return findings.map((finding) => `${finding.file}:${finding.line}`).sort()
}

export function toFileExports(matches: ExportedPropMemberMatch[]): string[] {
  return matches
    .map((match) => `${match.file}:${match.exportName}`)
    .sort()
}

export function toFiles(matches: Array<{ file: string }>): string[] {
  return [...new Set(matches.map((match) => match.file))].sort()
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

function collectInterfacePropertyEntries(node: ts.InterfaceDeclaration) {
  return node.members.flatMap((member) => {
    if (!ts.isPropertySignature(member)) {
      return []
    }

    const propName = getPropertyNameText(member.name)
    const typeText = member.type?.getText() ?? ""

    return propName ? [{ propName, typeText }] : []
  })
}

function collectTypePropertyEntries(node: ts.TypeNode): Array<{ propName: string; typeText: string }> {
  if (ts.isTypeLiteralNode(node)) {
    return node.members.flatMap((member) => {
      if (!ts.isPropertySignature(member)) {
        return []
      }

      const propName = getPropertyNameText(member.name)
      const typeText = member.type?.getText() ?? ""

      return propName ? [{ propName, typeText }] : []
    })
  }

  if (ts.isIntersectionTypeNode(node) || ts.isUnionTypeNode(node)) {
    return node.types.flatMap((typeNode) => collectTypePropertyEntries(typeNode))
  }

  if (ts.isParenthesizedTypeNode(node)) {
    return collectTypePropertyEntries(node.type)
  }

  return []
}

function getPropertyNameText(name: ts.PropertyName | ts.BindingName | undefined) {
  if (!name) {
    return null
  }

  if (ts.isIdentifier(name) || ts.isStringLiteral(name)) {
    return name.text
  }

  return null
}

function isExported(node: ts.Node) {
  if (!ts.canHaveModifiers(node)) {
    return false
  }

  return ts.getModifiers(node)?.some((modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword) ?? false
}

function visit(node: ts.Node, callback: (node: ts.Node) => void) {
  callback(node)
  ts.forEachChild(node, (child) => visit(child, callback))
}

function walk(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const fullPath = path.join(dir, entry.name)
    return entry.isDirectory() ? walk(fullPath) : [fullPath]
  })
}

function isSourceFile(file: string) {
  return /\.(ts|tsx)$/.test(file) && !/\.test\.(ts|tsx)$/.test(file)
}
