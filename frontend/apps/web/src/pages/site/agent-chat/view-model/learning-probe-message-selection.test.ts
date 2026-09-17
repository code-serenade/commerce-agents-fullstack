import { describe, expect, it } from "vitest"
import type { AgentChatMessage } from "../types"
import {
  getCurrentLearningProbeMessageId,
  getPersistedLearningProbeMessageId,
} from "./student-next-actions-view-model"

const completedAssistant: AgentChatMessage = {
  content: "Completed commerce guidance",
  id: "assistant-completed",
  role: "assistant",
}

describe("learning probe message selection", () => {
  it("offers a probe only on the latest completed assistant message", () => {
    expect(
      getCurrentLearningProbeMessageId([
        completedAssistant,
        { content: "Follow-up", id: "user-follow-up", role: "user" },
        {
          content: "Latest completed guidance",
          id: "assistant-latest",
          role: "assistant",
        },
      ])
    ).toBe("assistant-latest")
  })

  it("does not offer a historical probe while a newer turn is pending", () => {
    expect(
      getCurrentLearningProbeMessageId([
        completedAssistant,
        { content: "Follow-up", id: "user-follow-up", role: "user" },
      ])
    ).toBeNull()
    expect(
      getCurrentLearningProbeMessageId([
        completedAssistant,
        {
          content: "Partial response",
          id: "assistant-streaming",
          role: "assistant",
          status: "streaming",
        },
      ])
    ).toBeNull()
  })

  it("waits for a persisted UUID before querying the probe API", () => {
    expect(getPersistedLearningProbeMessageId(completedAssistant)).toBeNull()
    expect(
      getPersistedLearningProbeMessageId({
        ...completedAssistant,
        persistedId: "4dd12397-39f2-453c-a5fc-0816f50e0098",
      })
    ).toBe("4dd12397-39f2-453c-a5fc-0816f50e0098")
    expect(
      getPersistedLearningProbeMessageId({
        ...completedAssistant,
        id: "57cf7c60-1013-4c9f-abd8-f46a86e1a5d3",
      })
    ).toBe("57cf7c60-1013-4c9f-abd8-f46a86e1a5d3")
  })
})
