import { existsSync } from "node:fs"
import path from "node:path"
import { describe, expect, it } from "vitest"
import { findPrimitiveDirectories, getPrimitiveBaseName } from "./ast-helpers"

describe("ui-core primitive directory structure", () => {
  it("keeps the required primitive file set in every primitive directory", () => {
    const findings = findPrimitiveDirectories().flatMap((dir) => {
      const baseName = getPrimitiveBaseName(dir)
      const expectedFiles = [
        `${baseName}.tsx`,
        `${baseName}.types.ts`,
        `${baseName}.styles.ts`,
        "index.ts",
      ]

      return expectedFiles
        .filter((file) => !existsSync(path.join(dir, file)))
        .map((file) => `${path.relative(process.cwd(), dir)}/${file}`)
    })

    expect(findings).toEqual([])
  })
})
