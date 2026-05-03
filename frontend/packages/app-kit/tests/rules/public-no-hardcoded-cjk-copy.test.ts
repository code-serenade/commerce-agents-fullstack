import { describe, expect, it } from "vitest"
import {
  findPublicSourceFiles,
  findSourceFilesMatching,
  toFiles,
} from "./ast-helpers"

describe("app public i18n protocols", () => {
  it("does not embed hardcoded CJK copy in public source files", () => {
    const matches = findSourceFilesMatching(
      findPublicSourceFiles(),
      (sourceFile, sourceText) => {
        if (sourceFile.fileName.endsWith("/preferences/language-switch.tsx")) {
          return false
        }

        return /[\p{Script=Han}]/u.test(sourceText)
      }
    )

    expect(toFiles(matches)).toEqual([])
  })
})
