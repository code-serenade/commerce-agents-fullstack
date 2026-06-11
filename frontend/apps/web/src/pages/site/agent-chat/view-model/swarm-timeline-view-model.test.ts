import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { formatTimelineEvent } from "./swarm-timeline-view-model"

const t = ((key: string, options?: Record<string, string>) => {
  if (key === "agentChat.swarm.timeline.events.workspaceContextLoaded") {
    return `workspace ${options?.sources}`
  }
  return key
}) as never

describe("formatTimelineEvent", () => {
  it("formats safe workspace source count without source bodies", () => {
    const event: AgentRuntimeEvent = {
      type: "workspace_context_loaded",
      counts: {
        campaigns: 1,
        conversationFiles: 1,
      },
      readiness: "ready",
      sourceCount: 5,
    }

    expect(formatTimelineEvent(event, [], [], t)?.label).toBe("workspace 5")
  })
})
