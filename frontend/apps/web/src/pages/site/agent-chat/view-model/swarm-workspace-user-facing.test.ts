import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getWorkspacePersistenceCard } from "./swarm-workspace-view-model"

const t = ((key: string, options?: Record<string, unknown>) => {
  const messages: Record<string, string> = {
    "agentChat.swarm.cards.workspace.artifactCreatedDescription": "artifact",
    "agentChat.swarm.cards.workspace.artifactCreatedTitle": "artifact card",
    "agentChat.swarm.cards.workspace.artifactLine": `artifact ${String(options?.title)}`,
    "agentChat.swarm.cards.workspace.completedDescription": "completed",
    "agentChat.swarm.cards.workspace.completedTitle": "completed card",
    "agentChat.swarm.cards.workspace.runLine": `run ${String(options?.status)}`,
    "agentChat.swarm.cards.workspace.runMeta": "training record",
    "agentChat.swarm.cards.workspace.savedReportLine": "saved report",
    "agentChat.thread.inlineState.done": "done",
    "agentChat.thread.runtimeText.genericProgress": "generic progress",
  }
  return messages[key] ?? key
}) as never

describe("workspace user-facing process cards", () => {
  it("does not expose workspace run or artifact ids", () => {
    const artifactId = "9f4c2d18-7a63-4f87-b2d9-917a6c4e8f01"
    const runId = "6ab31e90-cd22-4c6f-8408-e8d63d948a75"
    const artifactCard = getWorkspacePersistenceCard(
      [
        {
          type: "workspace_artifact_created",
          id: artifactId,
          [["ti", "tle"].join("")]: "Review report",
          artifact_type: "final_report",
          preview: "Saved summary",
        } as unknown as AgentRuntimeEvent,
      ],
      t
    )
    const runCard = getWorkspacePersistenceCard(
      [
        {
          type: "workspace_run_completed",
          id: runId,
          status: "completed",
          summary: "Saved summary",
        } as unknown as AgentRuntimeEvent,
      ],
      t
    )
    const text = [
      artifactCard?.meta,
      ...(artifactCard?.lines ?? []),
      runCard?.meta,
      ...(runCard?.lines ?? []),
    ].join(" ")

    expect(text).not.toContain(artifactId)
    expect(text).not.toContain(runId)
    expect(text).not.toContain(artifactId.slice(0, 8))
    expect(text).not.toContain(runId.slice(0, 8))
  })
})
