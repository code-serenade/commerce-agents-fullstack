import { describe, expect, it } from "vitest"
import { isAgentShellRoute } from "./app-layout-routes"

describe("app layout routes", () => {
  it("keeps the materials library inside the agent chat shell", () => {
    expect(isAgentShellRoute("/materials")).toBe(true)
  })

  it("keeps auth pages outside the agent chat shell", () => {
    expect(isAgentShellRoute("/login")).toBe(false)
  })
})
