import { describe, expect, it } from "vitest"
import {
  findPublicSourceFiles,
  findSourceFilesMatching,
  toFiles,
} from "./ast-helpers"

describe("app public i18n protocols", () => {
  it("does not embed translation fallback copy in public source files", () => {
    const matches = findSourceFilesMatching(findPublicSourceFiles(), (_, sourceText) =>
      /\bt\(\s*["'`][^"'`]+["'`]\s*,\s*["'`][^"'`]+["'`]/.test(sourceText)
    )

    expect(toFiles(matches)).toEqual([])
  })
})
