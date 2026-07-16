import { describe, expect, it } from "vitest"
import type { WorkspaceArtifactResponse } from "@/api"
import { buildAgentOutputReport } from "./report-structure"

describe("buildAgentOutputReport", () => {
  it("maps a structured JSON artifact without using demo content", () => {
    const structuredContent = `{
      "title": "Summer apparel campaign",
      "description": "Improve conversion around commuter scenarios.",
      "prompt": "Plan an apparel campaign",
      "metrics": [{"label":"Target ROI","value":"1.8","description":"Seven-day paid ROI"}],
      "steps": [{"title":"Creative test","owner":"Ads","timing":"Days 1-2","detail":"Test three creative groups."}],
      "actions": ["Confirm inventory"],
      "sections": [{"title":"Conclusion","content":"Test creatives before scaling spend.","bullets":["Apply hourly stop-loss rules"]}]
    }`
    const originalTitle = "Original title"
    const artifact = createArtifact({
      content: structuredContent,
      format: "json",
      title: originalTitle,
    })

    const report = buildAgentOutputReport(artifact, null)

    expect(report?.title).toBe("Summer apparel campaign")
    expect(report?.prompt).toBe("Plan an apparel campaign")
    expect(report?.metrics[0]?.value).toBe("1.8")
    expect(report?.steps[0]?.title).toBe("Creative test")
    expect(report?.actions).toEqual(["Confirm inventory"])
    expect(report?.sections[0]?.content).toBe(
      "Test creatives before scaling spend."
    )
  })

  it("derives sections, metrics and actions from markdown", () => {
    const markdown = `# Repeat purchase report

This report organizes a repeat-purchase plan from the conversation.

| Metric | Target | Description |
| --- | --- | --- |
| Repeat purchase | +12% | 30-day repeat purchase |

## Conclusion

Segment members before sending differentiated benefits.

- Contact high-value members first
- Check unsubscribe rate weekly

## Next actions

- Export the member list
- Configure the first coupon
`
    const reportTitle = "Conversation report"
    const artifact = createArtifact({ content: markdown, title: reportTitle })
    const metricLabel = "Repeat purchase"
    const metricDescription = "30-day repeat purchase"

    const report = buildAgentOutputReport(artifact, null)

    expect(report?.title).toBe("Repeat purchase report")
    expect(report?.description).toContain("from the conversation")
    expect(report?.metrics).toEqual([
      {
        label: metricLabel,
        value: "+12%",
        description: metricDescription,
      },
    ])
    expect(report?.actions).toEqual([
      "Export the member list",
      "Configure the first coupon",
    ])
    expect(report?.sections.map((section) => section.title)).toEqual([
      "Conclusion",
      "Next actions",
    ])
  })

  it("uses export content and structured export metadata when available", () => {
    const oldBody = "Old body"
    const oldTitle = "Old title"
    const dynamicTitle = "Complete dynamic report"
    const platform = "Social channel"
    const artifact = createArtifact({ content: oldBody, title: oldTitle })
    const report = buildAgentOutputReport(artifact, {
      artifactId: artifact.id,
      content: "Complete exported body",
      contentChars: 22,
      fileName: "report.md",
      format: "markdown",
      metadata: { structuredReport: { title: dynamicTitle, platform } },
      mimeType: "text/markdown",
      sourceFormat: "markdown",
      sourceStorageKind: "inline",
      title: dynamicTitle,
      warnings: [],
      workspaceId: artifact.workspaceId,
    })

    expect(report?.title).toBe("Complete dynamic report")
    expect(report?.content).toBe("Complete exported body")
    expect(report?.badges).toContain("Social channel")
    expect(report?.fileName).toBe("report.md")
  })
})

function createArtifact(
  overrides: Partial<WorkspaceArtifactResponse>
): WorkspaceArtifactResponse {
  const defaultTitle = "Conversation report"
  return {
    artifactType: "final_report",
    content: null,
    format: "markdown",
    id: "artifact-1",
    source: "agent_session",
    status: "active",
    storageKind: "inline",
    title: defaultTitle,
    validationStatus: "unchecked",
    workspaceId: "workspace-1",
    ...overrides,
  }
}
