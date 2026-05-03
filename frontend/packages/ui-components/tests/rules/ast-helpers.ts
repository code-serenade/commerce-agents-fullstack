import { readFileSync, readdirSync } from "node:fs"
import path from "node:path"
import ts from "typescript"

const packageRoot = path.resolve(import.meta.dirname, "../..")
const stableRoot = path.join(packageRoot, "src/components/stable")

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

export function readPackageJson() {
  return JSON.parse(readFileSync(path.join(packageRoot, "package.json"), "utf8")) as {
    exports?: Record<string, string>
  }
}

export function findStableTsxFiles(): string[] {
  return walk(stableRoot).filter((file) => file.endsWith(".tsx"))
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

export function findCallExpressionFindings(
  files: string[],
  matcher: (node: ts.CallExpression) => boolean
): RuleFinding[] {
  return files.flatMap((file) => {
    const { sourceFile } = readSourceFile(file)
    const findings: RuleFinding[] = []

    visit(sourceFile, (node) => {
      if (ts.isCallExpression(node) && matcher(node)) {
        findings.push(toFinding(file, sourceFile, node))
      }
    })

    return findings
  })
}

export function findExportedPropMemberMatches(
  files: string[],
  matcher: (propName: string) => boolean
): ExportedPropMemberMatch[] {
  return files.flatMap((file) => {
    const { sourceFile } = readSourceFile(file)

    return sourceFile.statements.flatMap((statement) => {
      if (ts.isInterfaceDeclaration(statement) && isExported(statement)) {
        if (!statement.name.text.endsWith("Props")) {
          return []
        }

        return collectInterfacePropertyNames(statement)
          .filter(matcher)
          .map((propName) => ({
            file: path.relative(packageRoot, file),
            exportName: statement.name.text,
            propName,
          }))
      }

      if (ts.isTypeAliasDeclaration(statement) && isExported(statement)) {
        if (!statement.name.text.endsWith("Props")) {
          return []
        }

        return collectTypePropertyNames(statement.type)
          .filter(matcher)
          .map((propName) => ({
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

export function toFiles(findings: RuleFinding[]): string[] {
  return [...new Set(findings.map((finding) => finding.file))].sort()
}

export function toFileExports(
  matches: ExportedPropMemberMatch[]
): string[] {
  return matches
    .map((match) => `${match.file}:${match.exportName}`)
    .sort()
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

function collectInterfacePropertyNames(node: ts.InterfaceDeclaration): string[] {
  return node.members.flatMap((member) => {
    if (!ts.isPropertySignature(member)) {
      return []
    }

    const name = getPropertyNameText(member.name)
    return name ? [name] : []
  })
}

function collectTypePropertyNames(node: ts.TypeNode): string[] {
  if (ts.isTypeLiteralNode(node)) {
    return node.members.flatMap((member) => {
      if (!ts.isPropertySignature(member)) {
        return []
      }

      const name = getPropertyNameText(member.name)
      return name ? [name] : []
    })
  }

  if (ts.isIntersectionTypeNode(node) || ts.isUnionTypeNode(node)) {
    return node.types.flatMap((typeNode) => collectTypePropertyNames(typeNode))
  }

  if (ts.isParenthesizedTypeNode(node)) {
    return collectTypePropertyNames(node.type)
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
