import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import httpClient, { setHttpClientErrorHandler } from "./http-client"
import { RequestError } from "./errors/request-error"

describe("httpClient auth refresh", () => {
  beforeEach(() => {
    const store = new Map<string, string>()
    Object.defineProperty(globalThis, "localStorage", {
      configurable: true,
      value: {
        clear: () => store.clear(),
        getItem: (key: string) => (store.has(key) ? store.get(key)! : null),
        removeItem: (key: string) => store.delete(key),
        setItem: (key: string, value: string) => store.set(key, value),
      },
    })
  })

  afterEach(() => {
    setHttpClientErrorHandler(null)
    localStorage.clear()
    vi.unstubAllGlobals()
    vi.useRealTimers()
  })

  it("keeps stored tokens when refresh token service is temporarily unavailable", async () => {
    localStorage.setItem("token", "active-token")
    localStorage.setItem("refreshToken", "active-refresh")
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(new Response(null, { status: 401 }))
      .mockResolvedValueOnce(new Response(null, { status: 503 }))
    vi.stubGlobal("fetch", fetchMock)

    await expect(
      httpClient.request({
        method: "GET",
        url: "/api/app/me/permissions",
      })
    ).rejects.toMatchObject({
      response: {
        status: 503,
      },
    })

    expect(localStorage.getItem("token")).toBe("active-token")
    expect(localStorage.getItem("refreshToken")).toBe("active-refresh")
  })

  it("uses an explicit request token instead of the stored token", async () => {
    localStorage.setItem("token", "stored-token")
    const fetchMock = vi.fn().mockResolvedValue(
      Response.json({
        code: 0,
        data: { ok: true },
        message: "ok",
      })
    )
    vi.stubGlobal("fetch", fetchMock)

    await httpClient.request({
      method: "GET",
      token: "fresh-login-token",
      url: "/auth/user/me",
    })

    expect(fetchMock).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        headers: expect.objectContaining({
          Authorization: "Bearer fresh-login-token",
        }),
      })
    )
  })

  it("does not use the stored token when an explicit null token is provided", async () => {
    localStorage.setItem("token", "stored-token")
    const fetchMock = vi.fn().mockResolvedValue(
      Response.json({
        code: 0,
        data: { ok: true },
        message: "ok",
      })
    )
    vi.stubGlobal("fetch", fetchMock)

    await httpClient.request({
      method: "GET",
      token: null,
      url: "/auth/session/public",
    })

    expect(fetchMock).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        headers: expect.not.objectContaining({
          Authorization: expect.any(String),
        }),
      })
    )
  })

  it("does not refresh or clear the stored session when an explicit request token is rejected", async () => {
    localStorage.setItem("token", "stored-token")
    localStorage.setItem("refreshToken", "stored-refresh")
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(new Response(null, { status: 401 }))
    vi.stubGlobal("fetch", fetchMock)

    await expect(
      httpClient.request({
        method: "GET",
        token: "fresh-login-token",
        url: "/auth/user/me",
      })
    ).rejects.toMatchObject({
      response: { status: 401 },
    })

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(localStorage.getItem("token")).toBe("stored-token")
    expect(localStorage.getItem("refreshToken")).toBe("stored-refresh")
  })

  it("normalizes request timeout aborts and reports them through the global error handler", async () => {
    vi.useFakeTimers()
    const handledErrors: unknown[] = []
    setHttpClientErrorHandler((error) => handledErrors.push(error))
    vi.stubGlobal(
      "fetch",
      vi.fn((_url: string, init?: RequestInit) => {
        return new Promise((_resolve, reject) => {
          init?.signal?.addEventListener("abort", () => {
            reject(new DOMException("The operation was aborted.", "AbortError"))
          })
        })
      })
    )

    const requestPromise = httpClient.request({
      method: "GET",
      timeout: 100,
      url: "/api/app/me/permissions",
    })
    const rejection = expect(requestPromise).rejects.toMatchObject({
      kind: "network",
      message: "Request timed out",
    })

    await vi.advanceTimersByTimeAsync(100)

    await rejection
    expect(handledErrors).toHaveLength(1)
    expect(handledErrors[0]).toBeInstanceOf(RequestError)
    expect(handledErrors[0]).toMatchObject({
      kind: "network",
      message: "Request timed out",
    })
  })

  it("does not report timeout errors globally when the request suppresses global errors", async () => {
    vi.useFakeTimers()
    const errorHandler = vi.fn()
    setHttpClientErrorHandler(errorHandler)
    vi.stubGlobal(
      "fetch",
      vi.fn((_url: string, init?: RequestInit) => {
        return new Promise((_resolve, reject) => {
          init?.signal?.addEventListener("abort", () => {
            reject(new DOMException("The operation was aborted.", "AbortError"))
          })
        })
      })
    )

    const requestPromise = httpClient.request({
      method: "GET",
      suppressGlobalError: true,
      timeout: 100,
      url: "/api/app/me/permissions",
    })
    const rejection = expect(requestPromise).rejects.toBeInstanceOf(
      RequestError
    )

    await vi.advanceTimersByTimeAsync(100)

    await rejection
    expect(errorHandler).not.toHaveBeenCalled()
  })
})
