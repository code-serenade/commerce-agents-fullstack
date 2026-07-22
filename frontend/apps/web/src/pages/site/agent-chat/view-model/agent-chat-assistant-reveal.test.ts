import { describe, expect, it, vi } from "vitest"
import { createAssistantRevealController } from "./agent-chat-assistant-reveal"

describe("createAssistantRevealController", () => {
  it("renders each incoming delta without a character-by-character delay", async () => {
    const onVisibleContentChange = vi.fn()
    const controller = createAssistantRevealController({ onVisibleContentChange })

    controller.queue("First response chunk. ")
    controller.queue("Second response chunk.")

    expect(onVisibleContentChange).toHaveBeenNthCalledWith(
      1,
      "First response chunk. "
    )
    expect(onVisibleContentChange).toHaveBeenNthCalledWith(
      2,
      "First response chunk. Second response chunk."
    )
    expect(controller.getVisibleContent()).toBe(
      "First response chunk. Second response chunk."
    )
    await expect(controller.waitForIdle()).resolves.toBeUndefined()
  })
})
