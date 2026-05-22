import { describe, expect, it } from "vitest"
import { resolveAttachmentExtractionState } from "./agent-chat-attachment-extraction-state"
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

  it("prepares png images for vision input", async () => {
    const file = new File(["image"], "shelf.png", { type: "image/png" })

    const result = await createBrowserExtractionRequest(file)

    expect(result.readable).toBe(true)
    expect(result.imageDataUrl).toMatch(/^data:image\/png;base64,/)
    expect(result.request.status).toBe("extracted")
    expect(result.request.contentKind).toBe("image_vision_input")
    expect(result.request.parser).toBe("browser_image")
    expect(result.request.text).toContain("Image attachment prepared")
    expect(result.request.metadata).toMatchObject({
      sourceFileName: "shelf.png",
      visionDetail: "auto",
      visionReady: true,
    })
  })

  it("keeps server pending extraction visible instead of marking it readable", () => {
    expect(
      resolveAttachmentExtractionState({
        browserReadable: false,
        fallbackReason: "Browser text extraction supports plain text only.",
        serverExtraction: {
          contentKind: "unknown",
          fileId: "file-bin",
          jobId: "server-extraction-file-bin",
          message:
            "file is waiting for a text extraction adapter or trusted extraction upload",
          metadata: {},
          parser: "server_status_classifier",
          source: "server_job",
          status: "pending",
        },
        status: "unsupported",
      })
    ).toEqual({
      readable: false,
      status: "pending",
      unsupportedReason:
        "file is waiting for a text extraction adapter or trusted extraction upload",
    })
  })

  it("keeps server unsupported extraction visible instead of marking it readable", () => {
    expect(
      resolveAttachmentExtractionState({
        browserReadable: false,
        fallbackReason: "Browser text extraction supports plain text only.",
        serverExtraction: {
          contentKind: "pdf",
          fileId: "file-pdf",
          jobId: "server-extraction-file-pdf",
          message:
            "server parser is not enabled for this file type; file remains unavailable as readable text",
          metadata: {},
          parser: "server_status_classifier",
          source: "server_job",
          status: "unsupported",
        },
        status: "unsupported",
      })
    ).toEqual({
      readable: false,
      status: "unsupported",
      unsupportedReason:
        "server parser is not enabled for this file type; file remains unavailable as readable text",
    })
  })
})
