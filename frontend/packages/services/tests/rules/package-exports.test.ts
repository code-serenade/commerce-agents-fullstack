import { describe, expect, it } from "vitest"
import { fileExists, readPackageJson } from "./ast-helpers"

describe("services package exports", () => {
  it("uses the public export naming convention", () => {
    const exportsMap = readPackageJson().exports ?? {}
    const exportKeys = Object.keys(exportsMap).sort()

    const invalidKeys = exportKeys.filter(
      (key) => !/^\.\/[a-z0-9-]+(?:\/[a-z0-9-]+)*$/.test(key)
    )

    expect(invalidKeys).toEqual([])
  })

  it("points subpath exports to existing source files under src", () => {
    const exportsMap = readPackageJson().exports ?? {}

    const invalidEntries = Object.entries(exportsMap).flatMap(([key, value]) => {
      if (typeof value !== "string") {
        return [key]
      }

      const normalizedValue = value.replaceAll("\\", "/")
      const isSourceFile =
        normalizedValue.startsWith("./src/") &&
        (normalizedValue.endsWith(".ts") || normalizedValue.endsWith(".tsx"))

      return isSourceFile && fileExists(value) ? [] : [`${key} -> ${value}`]
    })

    expect(invalidEntries).toEqual([])
  })
})
