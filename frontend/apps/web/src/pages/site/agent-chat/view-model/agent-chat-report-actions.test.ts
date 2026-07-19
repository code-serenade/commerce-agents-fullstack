import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import {
  createAgentOutputReportUrl,
  getAgentChatReportRouteContext,
} from "./agent-chat-report-actions"

describe("agent chat report actions", () => {
  it("creates a report-center URL with conversation and latest run context", () => {
    const context = getAgentChatReportRouteContext({
      conversationId: "conversation-1",
      runtimeEvents: [
        {
          id: "run-1",
          status: "succeeded",
          type: "workspace_run_completed",
          workspaceId: "workspace-1",
        } as AgentRuntimeEvent,
      ],
    })

    expect(context).toEqual({
      conversationId: "conversation-1",
      runId: "run-1",
      workspaceId: "workspace-1",
    })
    expect(createAgentOutputReportUrl(context!)).toBe(
      "/agent-output?conversationId=conversation-1&workspaceId=workspace-1&runId=run-1"
    )
  })

  it("does not expose report actions before a conversation exists", () => {
    expect(
      getAgentChatReportRouteContext({
        conversationId: null,
        runtimeEvents: [],
      })
    ).toBeNull()
  })
})
