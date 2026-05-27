import { describe, expect, it } from "vitest"
import {
  fileKind,
  filterTrainingFiles,
  hasActiveTrainingFileFilters,
  summarizeTrainingFilesFilterResult,
} from "./logic"
import { file } from "./test-helpers"

describe("commerce files filter logic", () => {
  it("filters reusable files by name, type, status, object key, or upload time", () => {
    const files = [
      file({ id: "pdf", fileName: "pricing.pdf", mimeType: "application/pdf" }),
      file({
        id: "sheet",
        fileName: "sales.xlsx",
        mimeType: "application/vnd.ms-excel",
      }),
    ]

    expect(filterTrainingFiles(files, "xlsx").map((item) => item.id)).toEqual([
      "sheet",
    ])
    expect(filterTrainingFiles(files, "pricing").map((item) => item.id)).toEqual([
      "pdf",
    ])
  })

  it("filters reusable files by file kind and read status", () => {
    const files = [
      file({ id: "image", fileName: "hero.png", mimeType: "image/png" }),
      file({
        id: "sheet",
        fileName: "sales.csv",
        mimeType: "text/csv",
        metadata: { extractionStatus: "ready" },
      }),
      file({
        id: "word",
        fileName: "brief.docx",
        mimeType:
          "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        metadata: { extractionStatus: "failed" },
      }),
    ]

    expect(
      filterTrainingFiles(files, {
        keyword: "",
        kind: "sheet",
        readStatus: "readable",
      }).map((item) => item.id)
    ).toEqual(["sheet"])
    expect(
      filterTrainingFiles(files, {
        keyword: "",
        kind: "word",
        readStatus: "failed",
      }).map((item) => item.id)
    ).toEqual(["word"])
    expect(fileKind(files[0])).toBe("image")
  })

  it("classifies common upload material kinds by mime type or extension", () => {
    expect(fileKind(file({ fileName: "cover.webp", mimeType: "" }))).toBe(
      "image"
    )
    expect(fileKind(file({ fileName: "brief.pdf", mimeType: "" }))).toBe("pdf")
    expect(fileKind(file({ fileName: "script.docx", mimeType: "" }))).toBe(
      "word"
    )
    expect(fileKind(file({ fileName: "sales.xlsx", mimeType: "" }))).toBe(
      "sheet"
    )
    expect(fileKind(file({ fileName: "notes.md", mimeType: "" }))).toBe("text")
    expect(fileKind(file({ fileName: "archive.bin", mimeType: "" }))).toBe(
      "other"
    )
  })

  it("detects whether file filters are active", () => {
    expect(
      hasActiveTrainingFileFilters({
        keyword: "",
        kind: "all",
        readStatus: "all",
      })
    ).toBe(false)
    expect(
      hasActiveTrainingFileFilters({
        keyword: "pdf",
        kind: "all",
        readStatus: "all",
      })
    ).toBe(true)
    expect(
      hasActiveTrainingFileFilters({
        keyword: "",
        kind: "image",
        readStatus: "all",
      })
    ).toBe(true)
    expect(
      hasActiveTrainingFileFilters({
        keyword: "",
        kind: "all",
        readStatus: "failed",
      })
    ).toBe(true)
    expect(
      hasActiveTrainingFileFilters({
        keyword: "",
        kind: "all",
        readStatus: "all",
        usage: "selectedForTurn",
      })
    ).toBe(true)
  })

  it("summarizes current filter result counts", () => {
    const pdf = file({ id: "pdf", fileName: "pricing.pdf" })
    const sheet = file({
      id: "sheet",
      fileName: "sales.xlsx",
      mimeType: "application/vnd.ms-excel",
    })

    expect(
      summarizeTrainingFilesFilterResult({
        filters: {
          keyword: "pricing",
          kind: "all",
          readStatus: "all",
        },
        filteredFiles: [pdf],
        totalFiles: [pdf, sheet],
      })
    ).toEqual({
      active: true,
      filtered: 1,
      total: 2,
    })
    expect(
      summarizeTrainingFilesFilterResult({
        filters: {
          keyword: "",
          kind: "all",
          readStatus: "all",
        },
        filteredFiles: [pdf, sheet],
        totalFiles: [pdf, sheet],
      })
    ).toEqual({
      active: false,
      filtered: 2,
      total: 2,
    })
  })
})
