import { describe, expect, it } from "vitest"
import { getAgentChatComposerPanelMaxHeight } from "./use-agent-chat-composer-panel-layout"

describe("getAgentChatComposerPanelMaxHeight", () => {
  it("does not force a panel past the available viewport space", () => {
    expect(
      getAgentChatComposerPanelMaxHeight({
        availableHeight: 198,
        viewportLimit: 584,
      })
    ).toBe(198)
  })

  it("keeps the panel within its preferred maximum height", () => {
    expect(
      getAgentChatComposerPanelMaxHeight({
        availableHeight: 960,
        viewportLimit: 900,
      })
    ).toBe(640)
  })

  it("does not produce a negative height when the composer is outside the viewport", () => {
    expect(
      getAgentChatComposerPanelMaxHeight({
        availableHeight: -24,
        viewportLimit: 480,
      })
    ).toBe(0)
  })
})
