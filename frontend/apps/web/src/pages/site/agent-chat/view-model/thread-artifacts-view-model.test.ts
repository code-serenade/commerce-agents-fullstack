import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getInlineArtifactsFromRuntime } from "./thread-artifacts-view-model"

const t = ((key: string, params?: Record<string, unknown>) => {
  if (key === "agentChat.thread.artifacts.backendCreated") {
    return `${String(params?.type)}:${String(params?.status)}`
  }
  return key
}) as never

describe("getInlineArtifactsFromRuntime", () => {
  it("keeps backend artifact ids when repeated reports share title and type", () => {
    const title = ["Agent", " final report", " - hi"].join("")
    const artifactType = ["final", "_report"].join("")
    const events: AgentRuntimeEvent[] = [
      {
        type: "workspace_artifact_created",
        id: "artifact-a",
        title,
        status: "active",
        artifactType,
      },
      {
        type: "workspace_artifact_created",
        id: "artifact-b",
        title,
        status: "active",
        artifactType,
      },
    ]

    expect(getInlineArtifactsFromRuntime(events, t).map((item) => item.id)).toEqual([
      "artifact-a",
      "artifact-b",
    ])
  })
})
