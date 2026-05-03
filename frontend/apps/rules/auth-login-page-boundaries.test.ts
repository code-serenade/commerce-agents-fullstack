import { describe, expect, it } from "vitest"
import {
  appName,
  fileExists,
  findImportFindings,
  resolveAppPath,
  toLocations,
} from "./ast-helpers"

const loginFiles = [
  resolveAppPath("src/pages/auth/Login.tsx"),
  resolveAppPath("src/pages/public/Login.tsx"),
].filter(fileExists)

const forbiddenLoginImports = new Set([
  "react-hook-form",
  "@hookform/resolvers/zod",
  "zod",
  "@workspace/ui-components/stable/input",
  "@workspace/ui-components/stable/field",
])

describe(`${appName} auth login page boundaries`, () => {
  it("keeps form implementation inside app-kit login components", () => {
    const findings = findImportFindings(loginFiles, (node) => {
      const specifier = node.moduleSpecifier.getText().replace(/["']/g, "")
      return forbiddenLoginImports.has(specifier)
    })

    expect(toLocations(findings)).toEqual([])
  })
})
