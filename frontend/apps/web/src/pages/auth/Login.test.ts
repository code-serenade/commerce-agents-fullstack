import { describe, expect, it } from "vitest"
import { RequestError } from "@workspace/services/errors/request-error"
import { loginErrorCopyKey } from "./login-error-copy"

describe("loginErrorCopyKey", () => {
  it("points workspace 502 failures to local backend startup", () => {
    expect(
      loginErrorCopyKey(
        new RequestError("bad gateway", {
          kind: "api",
          status: 502,
        }),
        "workspace"
      )
    ).toBe("login.error.backendUnavailable")
  })

  it("keeps credential failures separate from backend availability", () => {
    expect(
      loginErrorCopyKey(
        new RequestError("unauthorized", {
          kind: "api",
          status: 401,
        }),
        "credentials"
      )
    ).toBe("login.error.invalidCredentials")
  })
})
