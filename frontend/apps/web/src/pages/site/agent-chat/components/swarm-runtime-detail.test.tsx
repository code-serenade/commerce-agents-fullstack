import { Bot } from "lucide-react"
import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import type { SwarmAgent } from "../view-model/swarm-panel-view-model"
import { ComputerRuntimeDetail } from "./swarm-runtime-detail"

const longObjective =
  "Read uploaded PDF Word Excel screenshots, check selected sources, compare search evidence, explain difficult terms for students, and verify every selected role contributes to the answer."
const agentName = "Data reviewer"

describe("ComputerRuntimeDetail", () => {
  it("wraps long business input and active agent objective instead of clamping them", async () => {
    const i18n = await createAgentChatTestI18n()
    const activeAgent = createAgent()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <ComputerRuntimeDetail
          activeAgent={activeAgent}
          activeAgentIndex={0}
          activeState="working"
          agents={[activeAgent]}
          runtimeEvents={[]}
        />
      </AgentChatTestI18nProvider>
    )

    const objectiveInstances = screen.getAllByText(longObjective)
    const businessInput = objectiveInstances[0]!
    const activeObjective = objectiveInstances.at(-1)!

    expect(businessInput).toHaveClass("wrap-break-word")
    expect(businessInput).not.toHaveClass("line-clamp-2")
    expect(activeObjective).toHaveClass("wrap-break-word")
    expect(activeObjective).not.toHaveClass("line-clamp-1")
  })

  it("shows final-answer completion without an internal role failure card", async () => {
    const i18n = await createAgentChatTestI18n()
    const activeAgent = createAgent()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <ComputerRuntimeDetail
          activeAgent={activeAgent}
          activeAgentIndex={0}
          activeState="done"
          agents={[activeAgent]}
          runtimeEvents={[
            {
              type: "task_node_failed",
              node_id: "design",
              reason: "design role output lacks design-specific content",
            },
            {
              type: "output",
              content: "Final answer body",
            },
          ]}
        />
      </AgentChatTestI18nProvider>
    )

    expect(screen.queryByText("Handling blocked")).toBeNull()
    expect(screen.queryByText("Experience design")).toBeNull()
    expect(screen.getByText("Done")).not.toBeNull()
  })
})

function createAgent(): SwarmAgent {
  return {
    dependsOn: [],
    icon: Bot,
    name: agentName,
    nodeId: "data",
    objective: longObjective,
    role: "data",
    summary: longObjective,
    tone: "bg-slate-100 text-slate-700",
  }
}
