import { afterEach, describe, expect, it, vi } from "vitest"
import { getAuthUserProfileApi } from "./auth"

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
})
