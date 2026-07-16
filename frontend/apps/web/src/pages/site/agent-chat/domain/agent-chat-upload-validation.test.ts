import { describe, expect, it } from "vitest"
import {
  AI_READABLE_FILE_ACCEPT,
  MAX_AI_READABLE_UPLOAD_BYTES,
  validateAiReadableUploads,
} from "./agent-chat-upload-validation"

describe("validateAiReadableUploads", () => {
  it("exposes only AI-readable formats to the file picker", () => {
    expect(AI_READABLE_FILE_ACCEPT.split(",")).toEqual([
      ".png",
      ".jpg",
      ".jpeg",
      ".webp",
      ".pdf",
      ".docx",
      ".xls",
      ".xlsx",
      ".csv",
      ".txt",
      ".md",
      ".markdown",
      ".json",
      ".tsv",
      ".html",
      ".htm",
    ])
    expect(AI_READABLE_FILE_ACCEPT).not.toContain(".gif")
    expect(AI_READABLE_FILE_ACCEPT).not.toContain(".doc,")
  })

  it.each([
    ["product.png", "image/png"],
    ["product.JPG", "image/jpeg"],
    ["brief.pdf", "application/pdf"],
    [
      "brief.docx",
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ],
    ["metrics.xls", "application/vnd.ms-excel"],
    [
      "metrics.xlsx",
      "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    ],
    ["metrics.csv", "text/csv"],
    ["notes.md", "text/markdown"],
    ["data.json", "application/json"],
    ["table.tsv", "text/tab-separated-values"],
    ["page.html", "text/html"],
  ])("accepts AI-readable file %s", (name, type) => {
    const file = new File(["content"], name, { type })

    expect(validateAiReadableUploads([file])).toEqual({
      acceptedFiles: [file],
      errors: [],
    })
  })

  it.each(["animation.gif", "legacy.doc", "slides.pptx", "archive.zip"])(
    "rejects unsupported format %s",
    (name) => {
      const file = new File(["content"], name)

      expect(validateAiReadableUploads([file]).errors[0]?.code).toBe(
        "unsupportedFormat"
      )
    }
  )

  it("rejects a file larger than the server request limit", () => {
    const file = new File(
      [new Uint8Array(MAX_AI_READABLE_UPLOAD_BYTES + 1)],
      "oversized.txt",
      { type: "text/plain" }
    )

    expect(validateAiReadableUploads([file]).errors[0]?.code).toBe("tooLarge")
  })

  it("rejects an empty file", () => {
    const file = new File([], "empty.txt", { type: "text/plain" })

    expect(validateAiReadableUploads([file]).errors[0]?.code).toBe("empty")
  })

  it("rejects a MIME type that conflicts with the extension", () => {
    const file = new File(["not a pdf"], "renamed.pdf", {
      type: "text/plain",
    })

    expect(validateAiReadableUploads([file]).errors[0]?.code).toBe(
      "mimeMismatch"
    )
  })

  it("keeps valid files when another selected file is invalid", () => {
    const valid = new File(["content"], "notes.txt", { type: "text/plain" })
    const invalid = new File(["content"], "slides.pptx")

    expect(validateAiReadableUploads([valid, invalid])).toEqual({
      acceptedFiles: [valid],
      errors: [{ code: "unsupportedFormat", file: invalid }],
    })
  })
})
