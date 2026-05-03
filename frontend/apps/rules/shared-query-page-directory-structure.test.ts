import path from "node:path"
import { describe, expect, it } from "vitest"
import {
  appName,
  fileExists,
  findPageDirectories,
  hasImport,
  toAppRelativePath,
} from "./ast-helpers"

const sharedQueryPages = findPageDirectories().filter((page) => {
  if (!page.dataFile) {
    return false
  }

  const pageUsesMetricsAndTable =
    hasImport(page.indexFile, "@workspace/app-kit") &&
    hasImport(page.dataFile, "./metrics") &&
    hasImport(page.dataFile, "./table/logic")

  return pageUsesMetricsAndTable
})

function expectedDataFileName(pageDir: string) {
  return `${path.basename(pageDir)}-data.tsx`
}

describe(`${appName} shared query page directory structure`, () => {
  it("keeps the approved template file set for mature shared query pages", () => {
    const missing = sharedQueryPages.flatMap((page) => {
      const requiredFiles = [
        "index.tsx",
        "types.ts",
        "constants.ts",
        expectedDataFileName(page.dir),
        "metrics/index.tsx",
        "table/index.ts",
        "table/logic.ts",
        "table/columns.tsx",
        "table/query-fields.ts",
        "table/status.ts",
        "table/sort.ts",
        "table/row-actions.ts",
      ]

      return requiredFiles
        .filter((file) => !fileExists(path.join(page.dir, file)))
        .map((file) => `${page.name}/${file}`)
    })

    expect(missing).toEqual([])
  })

  it("does not keep legacy root-level template files around", () => {
    const forbidden = sharedQueryPages.flatMap((page) => {
      const pageName = path.basename(page.dir)
      const forbiddenFiles = [
        `${pageName}-config.tsx`,
        `${pageName}-table.ts`,
        `${pageName}-table-logic.ts`,
        `${pageName}-row-actions.ts`,
        `${pageName}-metrics.tsx`,
        `use-${pageName}-table.ts`,
        "config",
      ]

      return forbiddenFiles
        .filter((file) => fileExists(path.join(page.dir, file)))
        .map((file) => toAppRelativePath(path.join(page.dir, file)))
    })

    expect(forbidden).toEqual([])
  })
})
