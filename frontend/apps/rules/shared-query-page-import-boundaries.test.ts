import { describe, expect, it } from "vitest"
import {
  appName,
  findImportFindings,
  findPageDirectories,
  getImportSpecifier,
  toLocations,
} from "./ast-helpers"

const pageDirectories = findPageDirectories()
const pageIndexFiles = pageDirectories.map((page) => page.indexFile)
const pageDataFiles = pageDirectories.flatMap((page) =>
  page.dataFile ? [page.dataFile] : []
)

describe(`${appName} shared query page import boundaries`, () => {
  it("keeps page indexes away from table internals", () => {
    const findings = findImportFindings(pageIndexFiles, (node) => {
      const specifier = getImportSpecifier(node)
      return specifier.startsWith("./table/") && specifier !== "./table"
    })

    expect(toLocations(findings)).toEqual([])
  })

  it("forces data layers to consume table logic directly instead of table entry", () => {
    const findings = findImportFindings(pageDataFiles, (node) => {
      return getImportSpecifier(node) === "./table"
    })

    expect(toLocations(findings)).toEqual([])
  })

  it("keeps data layers from depending on page dialogs", () => {
    const findings = findImportFindings(pageDataFiles, (node) => {
      return getImportSpecifier(node).startsWith("./dialogs/")
    })

    expect(toLocations(findings)).toEqual([])
  })
})
