import path from "node:path"
import { describe, expect, it } from "vitest"
import {
  appName,
  findSourceFiles,
  fileExists,
  resolveAppPath,
  toAppRelativePath,
} from "./ast-helpers"

const forbiddenGenericComponentNames = [
  "accordion",
  "alert",
  "avatar",
  "badge",
  "button",
  "card",
  "checkbox",
  "dialog",
  "drawer",
  "field",
  "form",
  "input",
  "modal",
  "pagination",
  "select",
  "sheet",
  "table",
  "tabs",
  "textarea",
  "toast",
  "tree",
]

describe(`${appName} local component boundaries`, () => {
  it("does not create a second generic component library in src/components", () => {
    const componentsRoot = resolveAppPath("src", "components")

    if (!fileExists(componentsRoot)) {
      expect([]).toEqual([])
      return
    }

    const genericFiles = findSourceFiles(componentsRoot)
      .filter((file) => {
        const name = path.basename(file).replace(/\.(ts|tsx)$/, "").toLowerCase()
        return forbiddenGenericComponentNames.includes(name)
      })
      .map(toAppRelativePath)
      .sort()

    expect(genericFiles).toEqual([])
  })
})
