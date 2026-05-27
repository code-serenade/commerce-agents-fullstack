import { describe, expect, it } from "vitest"
import { readTrainingFileSummary } from "./read-summary"
import { file } from "./test-helpers"

describe("commerce files read summary", () => {
  it("reads browser extraction metadata from top-level fields", () => {
    expect(
      readTrainingFileSummary(
        file({
          metadata: {
            contentKind: "markdown",
            originalTextChars: 3200,
            parser: "browser_text",
            summary: "Cold brew launch notes",
            truncated: true,
          },
        })
      )
    ).toEqual({
      contentKind: "markdown",
      failureReason: undefined,
      parser: "browser_text",
      summary: "Cold brew launch notes",
      textChars: 3200,
      truncated: true,
    })
  })

  it("reads nested server extraction metadata and failure reason", () => {
    expect(
      readTrainingFileSummary(
        file({
          metadata: {
            extraction: {
              contentKind: "document_text",
              failureReason: "parser failed",
              metadata: {
                originalTextChars: 6400,
              },
              parser: "document_text_extraction",
              textChars: 1800,
            },
          },
        })
      )
    ).toEqual({
      contentKind: "document_text",
      failureReason: "parser failed",
      parser: "document_text_extraction",
      summary: undefined,
      textChars: 1800,
      truncated: undefined,
    })
  })
})
