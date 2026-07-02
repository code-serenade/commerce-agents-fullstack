import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import type { DispatchWorkbenchModel } from "../view-model/agent-dispatch-workbench-view-model"
import { DispatchWorkbenchContent } from "./agent-dispatch-workbench-content"

const longRequest =
  "Diagnose this campaign with teaching mode, selected Data Ops Search Creative Accounting agents, attached file-a and file-b, and compare platform policy with competitor search evidence."
const longStepDetail =
  "Read selected roles, uploaded materials, search requirements, learning level, response mode, and verify whether every planned agent produced a matching runtime event."
const activeStepTitle = "Context and routing check"
const filename = "Multi-role process"
const internalFilename = "multi.agent.process.md"
const modeLabel = "Multi-agent collaboration process"
const progressLabel = "1/3 roles completed"
const progressStage = "Checking context and routing"

describe("DispatchWorkbenchContent", () => {
  it("wraps long request and active step details instead of clamping them", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <DispatchWorkbenchContent model={createWorkbenchModel()} />
      </AgentChatTestI18nProvider>
    )

    const request = screen.getByText(longRequest)
    const stepDetail = screen.getByText(longStepDetail)
    const previewPanel = screen.getByText(filename).closest(".rounded-md")

    expect(request).toHaveClass("wrap-break-word")
    expect(request).not.toHaveClass("line-clamp-2")
    expect(stepDetail).toHaveClass("wrap-break-word")
    expect(stepDetail).not.toHaveClass("line-clamp-1")
    expect(previewPanel?.querySelector(".overflow-y-auto")).toHaveClass("text-base/7")
    expect(screen.queryByText(internalFilename)).not.toBeInTheDocument()
  })
})

function createWorkbenchModel(): DispatchWorkbenchModel {
  return {
    activeStep: {
      detail: longStepDetail,
      id: "step-1",
      index: 0,
      state: "working",
      title: activeStepTitle,
    },
    activeStepIndex: 0,
    agents: [
      {
        name: "Data analyst",
        nodeId: "data",
        state: "working",
        status: "Working",
        summary: "Checks metrics and context",
      },
    ],
    facts: [],
    filename,
    kind: "multi_agent",
    modeLabel,
    previewLines: [
      "Quick answer task",
      "Business input: analyze this product",
      "Progress: 2/4 | Reading materials and preparing the answer",
    ],
    progress: {
      completed: 1,
      label: progressLabel,
      percent: 33,
      stage: progressStage,
      total: 3,
    },
    request: longRequest,
    steps: [
      {
        detail: longStepDetail,
        id: "step-1",
        index: 0,
        state: "working",
        title: activeStepTitle,
      },
    ],
    summary: modeLabel,
  }
}
