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
const filename = "multi.agent.process.md"
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

    expect(request).toHaveClass("wrap-break-word")
    expect(request).not.toHaveClass("line-clamp-2")
    expect(stepDetail).toHaveClass("wrap-break-word")
    expect(stepDetail).not.toHaveClass("line-clamp-1")
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
    facts: [],
    filename,
    kind: "multi_agent",
    modeLabel,
    previewLines: [],
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
