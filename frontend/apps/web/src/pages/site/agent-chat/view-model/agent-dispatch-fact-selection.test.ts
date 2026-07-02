import { describe, expect, it } from "vitest"
import { t } from "./agent-process-facts-test-helpers"
import { selectDispatchFacts } from "./agent-dispatch-fact-selection"

describe("selectDispatchFacts", () => {
  it("does not expose untranslated i18n keys in process facts", () => {
    const detail = "agentChat.thread.dispatchFacts.roleAudit.warning"
    const label = "agentChat.thread.dispatchFacts.roleAudit.label"
    const expectedDetail = "Role status synced"
    const expectedLabel = "Role check"
    const facts = selectDispatchFacts(
      [
        {
          detail,
          label,
          tone: "warning",
        },
      ],
      t
    )

    expect(facts).toEqual([
      {
        detail: expectedDetail,
        label: expectedLabel,
        tone: "warning",
      },
    ])
  })
})
