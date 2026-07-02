import { describe, expect, it } from "vitest"
import { isAgentShellRoute } from "./app-layout-routes"

describe("app layout routes", () => {
  it("keeps the materials library inside the agent chat shell", () => {
    expect(isAgentShellRoute("/materials")).toBe(true)
  })

  it("keeps child paths of agent-context pages inside the agent chat shell", () => {
    expect(isAgentShellRoute("/materials/")).toBe(true)
    expect(isAgentShellRoute("/materials/source-1")).toBe(true)
    expect(isAgentShellRoute("/agent-capabilities/")).toBe(true)
    expect(isAgentShellRoute("/question-ideas/templates")).toBe(true)
    expect(isAgentShellRoute("/overview")).toBe(true)
    expect(isAgentShellRoute("/products")).toBe(true)
    expect(isAgentShellRoute("/campaigns")).toBe(true)
    expect(isAgentShellRoute("/agent-output")).toBe(true)
  })

  it("keeps auth pages outside the agent chat shell", () => {
    expect(isAgentShellRoute("/login")).toBe(false)
    expect(isAgentShellRoute("/login/reset")).toBe(false)
  })
})
