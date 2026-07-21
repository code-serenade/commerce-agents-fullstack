import { afterEach, describe, expect, it, vi } from "vitest"
import { uploadDocumentToFileServiceApi } from "@workspace/services/api/file"

describe("document upload compatibility", () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it("transports CSV bytes through the accepted text document route", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      Response.json({
        code: 0,
        data: {
          already_uploaded: false,
          key: "users/test/docs/context.txt",
          sha256: "csv-sha",
          size: 12,
          url: "https://files.example/context.txt",
        },
        message: "ok",
      })
    )
    vi.stubGlobal("fetch", fetchMock)

    await expect(
      uploadDocumentToFileServiceApi(
        new File(["metric,value\nbudget,8888"], "context.csv", {
          type: "text/csv",
        })
      )
    ).resolves.toMatchObject({
      key: "users/test/docs/context.txt",
      sha256: "csv-sha",
    })

    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect(url).toBe("/file/sign/upload/document?ext=txt")
    expect(init).toMatchObject({ method: "PUT" })
    expect(init.body).toBeInstanceOf(Blob)
    expect((init.body as Blob).type).toBe("text/plain")
    await expect((init.body as Blob).text()).resolves.toBe(
      "metric,value\nbudget,8888"
    )
  })
})
