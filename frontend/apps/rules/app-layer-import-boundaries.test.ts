import { describe, expect, it } from "vitest"
import {
  appName,
  findAppSourceFiles,
  findImportFindings,
  getImportSpecifier,
  toLocations,
} from "./ast-helpers"

describe(`${appName} app layer import boundaries`, () => {
  it("does not import ui-core directly from app source", () => {
    const findings = findImportFindings(findAppSourceFiles(), (node) => {
      return getImportSpecifier(node).startsWith("@workspace/ui-core")
    })

    expect(toLocations(findings)).toEqual([])
  })
})
