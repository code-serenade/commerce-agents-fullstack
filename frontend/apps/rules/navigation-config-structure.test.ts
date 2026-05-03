import path from "node:path"
import { describe, expect, it } from "vitest"
import ts from "typescript"
import { appName, readSourceFile, resolveAppPath } from "./ast-helpers"

const navigationConfigFile = resolveAppPath("src/navigation/menu-config.tsx")

interface NavigationIdFinding {
  kind: "section" | "item" | "subItem"
  line: number
}

function findNavigationSectionsInitializer(sourceFile: ts.SourceFile) {
  for (const statement of sourceFile.statements) {
    if (!ts.isVariableStatement(statement)) {
      continue
    }

    for (const declaration of statement.declarationList.declarations) {
      if (
        ts.isIdentifier(declaration.name) &&
        /navigationSections$/i.test(declaration.name.text) &&
        declaration.initializer &&
        ts.isArrayLiteralExpression(declaration.initializer)
      ) {
        return declaration.initializer
      }
    }
  }

  return null
}

function getObjectProperty(
  node: ts.ObjectLiteralExpression,
  name: string
): ts.PropertyAssignment | null {
  for (const property of node.properties) {
    if (
      ts.isPropertyAssignment(property) &&
      ts.isIdentifier(property.name) &&
      property.name.text === name
    ) {
      return property
    }
  }

  return null
}

function hasStableStringId(node: ts.ObjectLiteralExpression) {
  const idProperty = getObjectProperty(node, "id")
  if (!idProperty) {
    return false
  }

  return (
    ts.isStringLiteral(idProperty.initializer) &&
    idProperty.initializer.text.trim().length > 0
  )
}

function getObjectArrayProperty(
  node: ts.ObjectLiteralExpression,
  name: string
): ts.ObjectLiteralExpression[] {
  const property = getObjectProperty(node, name)

  if (!property || !ts.isArrayLiteralExpression(property.initializer)) {
    return []
  }

  return property.initializer.elements.filter(ts.isObjectLiteralExpression)
}

function toFinding(
  sourceFile: ts.SourceFile,
  node: ts.ObjectLiteralExpression,
  kind: NavigationIdFinding["kind"]
): NavigationIdFinding {
  const { line } = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile))

  return {
    kind,
    line: line + 1,
  }
}

function findMissingIdFindings(sourceFile: ts.SourceFile) {
  const navigationSections = findNavigationSectionsInitializer(sourceFile)

  if (!navigationSections) {
    return [
      {
        kind: "section" as const,
        line: 1,
      },
    ]
  }

  const findings: NavigationIdFinding[] = []
  const sections = navigationSections.elements.filter(ts.isObjectLiteralExpression)

  for (const section of sections) {
    if (!hasStableStringId(section)) {
      findings.push(toFinding(sourceFile, section, "section"))
    }

    for (const item of getObjectArrayProperty(section, "items")) {
      if (!hasStableStringId(item)) {
        findings.push(toFinding(sourceFile, item, "item"))
      }

      for (const subItem of getObjectArrayProperty(item, "subItems")) {
        if (!hasStableStringId(subItem)) {
          findings.push(toFinding(sourceFile, subItem, "subItem"))
        }
      }
    }
  }

  return findings
}

describe(`${appName} navigation config structure`, () => {
  it("requires stable string ids for every section, item, and sub-item", () => {
    const { sourceFile } = readSourceFile(navigationConfigFile)
    const configPath = path.relative(process.cwd(), navigationConfigFile)
    const findings = findMissingIdFindings(sourceFile).map(
      (finding) => `${finding.kind}@${configPath}:${finding.line}`
    )

    expect(findings).toEqual([])
  })
})
