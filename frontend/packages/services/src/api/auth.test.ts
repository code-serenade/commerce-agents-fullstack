import { afterEach, describe, expect, it, vi } from "vitest"
import {
  REGISTER_PASSWORD_MAX_LENGTH,
  getAuthUserProfileApi,
  isRegisterPasswordValid,
  meApi,
} from "./auth"
import { setHttpClientErrorHandler } from "../http-client"

describe("auth api", () => {
  afterEach(() => {
    vi.unstubAllGlobals()
    setHttpClientErrorHandler(null)
  })

  it("loads the auth user profile with the provided access token", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      Response.json({
        code: 0,
        data: {
          id: "1",
          username: "admin",
          display_name: "Admin",
          email_verified: true,
          disabled: false,
        },
        message: "ok",
      })
    )
    vi.stubGlobal("fetch", fetchMock)

    await expect(getAuthUserProfileApi("fresh-access-token")).resolves.toEqual({
      id: "1",
      username: "admin",
      display_name: "Admin",
      email_verified: true,
      disabled: false,
    })

    expect(fetchMock).toHaveBeenCalledWith(
      "/auth/user/me",
      expect.objectContaining({
        headers: expect.objectContaining({
          Authorization: "Bearer fresh-access-token",
        }),
      })
    )
  })

  it("can suppress the global error banner while restoring a session", async () => {
    const globalErrorHandler = vi.fn()
    vi.stubGlobal("localStorage", {
      getItem: vi.fn().mockReturnValue(null),
    })
    const fetchMock = vi
      .fn()
      .mockResolvedValue(
        Response.json({ code: 503, message: "unavailable" }, { status: 503 })
      )
    vi.stubGlobal("fetch", fetchMock)
    setHttpClientErrorHandler(globalErrorHandler)

    await expect(meApi({ suppressGlobalError: true })).rejects.toBeDefined()

    expect(fetchMock).toHaveBeenCalledWith(
      "/auth/user/me",
      expect.objectContaining({ method: "GET" })
    )
    expect(globalErrorHandler).not.toHaveBeenCalled()
  })

  it("validates registration passwords with length, allowed characters, and two-category requirements", () => {
    const maxLengthPassword = `A1${"a".repeat(REGISTER_PASSWORD_MAX_LENGTH - 2)}`

    expect(isRegisterPasswordValid("Abcdef1!")).toBe(true)
    expect(isRegisterPasswordValid(` ${maxLengthPassword} `)).toBe(true)
    expect(isRegisterPasswordValid("abcdef12")).toBe(true)
    expect(isRegisterPasswordValid("abcdef!!")).toBe(true)
    expect(isRegisterPasswordValid("1234567!")).toBe(true)
    expect(isRegisterPasswordValid("Abcd123！")).toBe(true)
    expect(isRegisterPasswordValid("Abcd123_")).toBe(true)
    expect(isRegisterPasswordValid("Abc123中文!")).toBe(false)
    expect(isRegisterPasswordValid("Abc 123!")).toBe(false)
    expect(isRegisterPasswordValid("Abc12")).toBe(false)
    expect(isRegisterPasswordValid(`${maxLengthPassword}a`)).toBe(false)
    expect(isRegisterPasswordValid("abcdefgh")).toBe(false)
    expect(isRegisterPasswordValid("12345678")).toBe(false)
    expect(isRegisterPasswordValid("!!!!!!!!")).toBe(false)
  })
})
