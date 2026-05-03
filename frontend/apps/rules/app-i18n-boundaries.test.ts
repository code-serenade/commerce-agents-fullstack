import { describe, expect, it } from "vitest"
import ts from "typescript"
import {
  appName,
  findAppSourceFiles,
  findNodeFindings,
  toLocations,
} from "./ast-helpers"

const runtimeSourceFiles = findAppSourceFiles().filter(
  (file) => !file.includes("/src/test/") && !file.endsWith("/src/test/setup.ts")
)
const userCopyPropertyNames = new Set([
  "aria-label",
  "closeLabel",
  "description",
  "emptyLabel",
  "label",
  "markAllLabel",
  "meta",
  "panelTitle",
  "placeholder",
  "title",
])

function hasCjkCopy(text: string) {
  return /[\p{Script=Han}]/u.test(text)
}

function isStringLike(node: ts.Node): node is ts.StringLiteral | ts.NoSubstitutionTemplateLiteral {
  return ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)
}

function getPropertyNameText(name: ts.PropertyName) {
  if (ts.isIdentifier(name) || ts.isStringLiteral(name)) {
    return name.text
  }

  return null
}

function isUserCopyPropertyAssignment(node: ts.Node) {
  if (!ts.isPropertyAssignment(node)) {
    return false
  }

  const propertyName = getPropertyNameText(node.name)

  return propertyName ? userCopyPropertyNames.has(propertyName) : false
}

function isUserCopyJsxAttribute(node: ts.Node) {
  if (!ts.isJsxAttribute(node)) {
    return false
  }

  if (!ts.isIdentifier(node.name)) {
    return false
  }

  return userCopyPropertyNames.has(node.name.text)
}

function getUserCopyPropertyAssignment(node: ts.Node) {
  return isUserCopyPropertyAssignment(node) && ts.isPropertyAssignment(node)
    ? node
    : null
}

function getUserCopyJsxAttribute(node: ts.Node) {
  return isUserCopyJsxAttribute(node) && ts.isJsxAttribute(node) ? node : null
}

function isTranslationCallWithFallback(node: ts.Node) {
  if (!ts.isCallExpression(node)) {
    return false
  }

  if (!ts.isIdentifier(node.expression) || node.expression.text !== "t") {
    return false
  }

  const fallbackArgument = node.arguments[1]

  return Boolean(fallbackArgument && isStringLike(fallbackArgument))
}

describe(`${appName} app i18n boundaries`, () => {
  it("does not embed hardcoded CJK copy in runtime source", () => {
    const findings = findNodeFindings(runtimeSourceFiles, (node) => {
      return isStringLike(node) && hasCjkCopy(node.text)
    })

    expect(toLocations(findings)).toEqual([])
  })

  it("does not pass fallback copy through translation calls", () => {
    const findings = findNodeFindings(runtimeSourceFiles, (node) => {
      return isTranslationCallWithFallback(node)
    })

    expect(toLocations(findings)).toEqual([])
  })

  it("does not use i18n defaultValue fallbacks in app runtime source", () => {
    const findings = findNodeFindings(runtimeSourceFiles, (node) => {
      if (!ts.isPropertyAssignment(node)) {
        return false
      }

      const propertyName = getPropertyNameText(node.name)

      return propertyName === "defaultValue"
    })

    expect(toLocations(findings)).toEqual([])
  })

  it("keeps common user-facing copy properties behind locale", () => {
    const findings = findNodeFindings(runtimeSourceFiles, (node) => {
      const propertyAssignment = getUserCopyPropertyAssignment(node)
      if (propertyAssignment && isStringLike(propertyAssignment.initializer)) {
        return true
      }

      const jsxAttribute = getUserCopyJsxAttribute(node)
      if (jsxAttribute?.initializer && isStringLike(jsxAttribute.initializer)) {
        return true
      }

      return false
    })

    expect(toLocations(findings)).toEqual([])
  })
})
