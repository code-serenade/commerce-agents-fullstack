import { renderHook } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { savePendingRegistrationProfile } from "@/api"
import { useRegisterPage } from "./use-register-page"

vi.mock("@/analytics/product-analytics", () => ({
  trackProductEvent: vi.fn(),
}))

vi.mock("@/api", () => ({
  savePendingRegistrationProfile: vi.fn(),
}))

describe("useRegisterPage", () => {
  it("saves registration profile without extra school and team fields", () => {
    const { result } = renderHook(() => useRegisterPage())

    result.current.onRegistered?.({
      username: "student",
      password: "password123",
      displayName: "",
      email: "",
      extraFields: {},
    })

    expect(savePendingRegistrationProfile).toHaveBeenCalledWith({
      username: "student",
      displayName: "",
      email: "",
      schoolName: "",
      teamName: "",
    })
  })
})
