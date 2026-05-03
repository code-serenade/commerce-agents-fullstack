import path from "node:path"
import { describe, expect, it } from "vitest"
import {
  appName,
  findSourceFiles,
  getExistingPageRoots,
  toAppRelativePath,
} from "./ast-helpers"

const allowedSupportDirectories = new Set([
  "charts",
  "components",
  "dialogs",
  "metrics",
  "table",
  "tree",
])
const allowedStandalonePageFiles = new Set(["Login.tsx"])
const allowedPageSupportFilePatterns = [
  /-data\.tsx$/,
  /-data\.ts$/,
  /^constants\.ts$/,
  /^types\.ts$/,
  /^use-[a-z0-9-]+-page\.ts$/,
]

function isInsideSupportDirectory(file: string, pageRoot: string) {
  const segments = path.relative(pageRoot, file).split(path.sep)

  return segments.some((segment) => allowedSupportDirectories.has(segment))
}

describe(`${appName} page directory boundaries`, () => {
  it("keeps pages as directories with index.tsx entries", () => {
    const pageFiles = getExistingPageRoots().flatMap((root) =>
      findSourceFiles(root).filter((file) => file.endsWith(".tsx"))
    )

    const standaloneFiles = pageFiles.filter((file) => {
      const fileName = path.basename(file)
      const parentName = path.basename(path.dirname(file))

      if (fileName === "index.tsx") {
        return false
      }

      if (allowedStandalonePageFiles.has(fileName)) {
        return false
      }

      if (
        allowedPageSupportFilePatterns.some((pattern) => pattern.test(fileName))
      ) {
        return false
      }

      if (parentName === "lazy") {
        return false
      }

      return !getExistingPageRoots().some((root) =>
        isInsideSupportDirectory(file, root)
      )
    })

    expect(standaloneFiles.map(toAppRelativePath).sort()).toEqual([])
  })
})
