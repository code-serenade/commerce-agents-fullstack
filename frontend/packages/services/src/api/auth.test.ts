import { afterEach, describe, expect, it, vi } from "vitest"
import {
  REGISTER_PASSWORD_MAX_LENGTH,
  getAuthUserProfileApi,
  isRegisterPasswordValid,
} from "./auth"

describe("auth api", () => {
  afterEach(() => {
    vi.unstubAllGlobals()
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
