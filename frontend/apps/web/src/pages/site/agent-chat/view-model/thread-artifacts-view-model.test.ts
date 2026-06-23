import { describe, expect, it } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { getInlineArtifactsFromRuntime } from "./thread-artifacts-view-model"

const fallbackFilename = "stage material"
const referenceMeta = "reference"
const referenceStatus = "reference saved"

const t = ((key: string, params?: Record<string, unknown>) => {
  if (key === "agentChat.thread.artifacts.backendCreated") {
    return `${String(params?.type)}:${String(params?.status)}`
  }
  if (key === "agentChat.thread.artifacts.fileFallback") {
    return fallbackFilename
  }
  if (key === "agentChat.thread.artifacts.finalReportMeta") {
    return "review"
  }
  if (key === "agentChat.thread.artifacts.genericMeta") {
    return "material"
  }
  if (key === "agentChat.thread.artifacts.searchEvidence") {
    return referenceStatus
  }
  if (key === "agentChat.thread.artifacts.searchEvidenceMeta") {
    return referenceMeta
  }
  if (key === "agentChat.thread.artifacts.statusReady") {
    return "synced"
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

  it("does not expose backend artifact internals in visible cards", () => {
    const internalType = ["search", "evidence"].join("_")
    const internalTitle = ["worker", "search"].join(".")
    const internalPreview = ["request", "id=req-1"].join("_")
    const artifactEvent = {
      type: "workspace_artifact_created",
      id: "artifact-search",
      title: internalTitle,
      status: "active",
      artifactType: internalType,
      preview: internalPreview,
    } satisfies AgentRuntimeEvent
    const events: AgentRuntimeEvent[] = [
      artifactEvent,
    ]

    const [artifact] = getInlineArtifactsFromRuntime(events, t)
    const visibleText = JSON.stringify(artifact).toLowerCase()

    expect(artifact?.filename).toBe(fallbackFilename)
    expect(artifact?.meta).toBe(referenceMeta)
    expect(artifact?.status).toBe(referenceStatus)
    expect(visibleText).not.toContain("worker")
    expect(visibleText).not.toContain(internalType)
    expect(visibleText).not.toContain("request_id")
  })
})
