import { describe, expect, it } from "vitest"
import {
  applyRequestedComposerPanelFromSearch,
  clearComposerPanelSearchParam,
  requestedComposerPanelFromSearch,
} from "./agent-chat-composer-panel-url"

describe("agent chat composer panel url", () => {
  it("opens the source panel for the materials panel query", () => {
    expect(
      requestedComposerPanelFromSearch(
        new URLSearchParams("panel=materials&from=files")
      )
    ).toBe("source")
  })

  it("clears only the composer panel query param", () => {
    const searchParams = clearComposerPanelSearchParam(
      new URLSearchParams("panel=materials&from=files")
    )

    expect(searchParams.toString()).toBe("from=files")
  })

  it("increments panel request state when a panel query is present", () => {
    expect(
      applyRequestedComposerPanelFromSearch(
        { focusVersion: 2, panel: null, version: 3 },
        new URLSearchParams("panel=materials")
      )
    ).toEqual({
      focusVersion: 3,
      panel: "source",
      version: 4,
    })
  })
})
