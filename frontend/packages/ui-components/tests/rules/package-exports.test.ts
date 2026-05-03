import { describe, expect, it } from "vitest"
import { readPackageJson } from "./ast-helpers"

describe("ui-components package exports", () => {
  it("only exposes the approved public export categories", () => {
    const exportsMap = readPackageJson().exports ?? {}
    const exportKeys = Object.keys(exportsMap)
    const legacyKeys = exportKeys.filter(
      (key) => ![".", "./styles.css", "./stable/*", "./labs/*"].includes(key)
    )

    expect(legacyKeys.sort()).toEqual([])
  })
})
