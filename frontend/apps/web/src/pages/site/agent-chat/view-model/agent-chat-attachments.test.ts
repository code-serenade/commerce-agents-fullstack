import { describe, expect, it } from "vitest"
import { createBrowserExtractionRequest } from "./agent-chat-attachments"

describe("agent chat attachment extraction", () => {
  it("creates extracted metadata for readable text files", async () => {
    const file = new File([" Product note: low sugar cold brew "], "coffee.md", {
      type: "text/markdown",
    })

    const result = await createBrowserExtractionRequest(file)

    expect(result.readable).toBe(true)
    expect(result.request.status).toBe("extracted")
    expect(result.request.contentKind).toBe("markdown")
    expect(result.request.parser).toBe("browser_text")
    expect(result.request.source).toBe("browser_upload")
    expect(result.request.text).toBe("Product note: low sugar cold brew")
    expect(result.request.metadata).toMatchObject({
      sourceFileName: "coffee.md",
      truncated: false,
    })
  })

  it("marks empty text files as failed extraction without failing upload", async () => {
    const file = new File(["   \n  "], "empty.txt", { type: "text/plain" })

    const result = await createBrowserExtractionRequest(file)

    expect(result.readable).toBe(false)
    expect(result.request.status).toBe("failed")
    expect(result.request.failureReason).toContain("no readable text")
    expect(result.request.text).toBeUndefined()
  })

  it("marks binary files as unsupported extraction", async () => {
    const file = new File(["fake"], "brief.pdf", { type: "application/pdf" })

    const result = await createBrowserExtractionRequest(file)

    expect(result.readable).toBe(false)
    expect(result.request.status).toBe("unsupported")
    expect(result.request.contentKind).toBe("unsupported")
    expect(result.request.failureReason).toContain("plain text")
  })
})
