import { describe, expect, it } from "vitest"
import { formatTrainingFileMetadata } from "./metadata-json"
import { file } from "./test-helpers"

describe("commerce files metadata json", () => {
  it("formats empty metadata as an empty object", () => {
    expect(formatTrainingFileMetadata(file({ metadata: {} }))).toBe("{}")
  })

  it("formats nested extraction metadata for inspection", () => {
    expect(
      formatTrainingFileMetadata(
        file({
          metadata: {
            extraction: {
              metadata: {
                originalTextChars: 1200,
                truncated: true,
              },
              parser: "document_text_extraction",
              status: "extracted",
            },
          },
        })
      )
    ).toContain('"originalTextChars": 1200')
  })
})
