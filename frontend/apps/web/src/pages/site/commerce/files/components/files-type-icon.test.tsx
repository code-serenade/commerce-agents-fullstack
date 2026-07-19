import { describe, expect, it } from "vitest"
import { file } from "../domain/test-helpers"
import { filesIconKind } from "./files-type-icon-kind"

describe("filesIconKind", () => {
  it.each([
    ["image/png", "hero.png", "image"],
    ["application/pdf", "brief.pdf", "document"],
    [
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
      "brief.docx",
      "document",
    ],
    ["text/csv", "sales.csv", "sheet"],
    ["text/plain", "notes.txt", "text"],
    ["application/octet-stream", "archive.bin", "document"],
  ] as const)("maps %s to the %s icon group", (mimeType, fileName, expected) => {
    expect(filesIconKind(file({ fileName, mimeType }))).toBe(expected)
  })
})
