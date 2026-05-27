import { act, renderHook, waitFor } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { agentChatData } from "../agent-chat-data"
import { useAgentRuntimeOptions } from "./use-agent-runtime-options"

vi.mock("../agent-chat-data", () => ({
  agentChatData: {
    getRuntimeOptions: vi.fn(),
  },
}))

describe("useAgentRuntimeOptions", () => {
  it("keeps readable fallback runtime options when the backend config is unavailable", async () => {
    vi.mocked(agentChatData.getRuntimeOptions).mockRejectedValue(
      new Error("runtime options unavailable")
    )

    const { result } = renderHook(() => useAgentRuntimeOptions({ enabled: true }))

    await waitFor(() => {
      expect(result.current.isLoadingRuntimeOptions).toBe(false)
    })

    expect(result.current.runtimeOptions?.routingModes.map((mode) => mode.id))
      .toEqual(["commander", "fast"])
    expect(
      result.current.runtimeOptions?.collaborationModes.map((mode) => mode.id)
    ).toEqual(["auto", "single", "manual"])

    act(() => {
      result.current.setCollaborationMode("single")
      result.current.setPrimaryRole("data")
      result.current.setRoutingMode("fast")
    })

    expect(result.current.runtimeSelection).toMatchObject({
      collaborationMode: "single",
      primaryRole: "data",
      routingMode: "fast",
      selectedRoles: ["data"],
    })
  })
})
