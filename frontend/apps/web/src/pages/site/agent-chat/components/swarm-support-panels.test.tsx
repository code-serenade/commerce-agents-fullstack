import { render, screen } from "@testing-library/react"
import { MemoryRouter } from "react-router"
import { describe, expect, it } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import {
  SwarmIdleDetail,
  WorkspaceHandoffRail,
} from "./swarm-support-panels"

const longTask =
  "Use teaching mode with selected Data Ops Search Creative Accounting agents, read uploaded Word Excel PDF screenshots, compare current platform policy search evidence, and explain each difficult term for students."

describe("SwarmIdleDetail", () => {
  it("wraps long business input instead of clamping it", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <SwarmIdleDetail hasTask isStreaming={false} lastTask={longTask} />
      </AgentChatTestI18nProvider>
    )

    const task = screen.getByText(longTask)

    expect(task).toHaveClass("wrap-break-word")
    expect(task).not.toHaveClass("line-clamp-2")
  })
})

describe("WorkspaceHandoffRail", () => {
  it("wraps handoff details instead of clamping them", async () => {
    const i18n = await createAgentChatTestI18n()
    const detail = i18n.t("agentChat.swarm.handoffs.context.detail")

    render(
      <MemoryRouter>
        <AgentChatTestI18nProvider i18n={i18n}>
          <WorkspaceHandoffRail />
        </AgentChatTestI18nProvider>
      </MemoryRouter>
    )

    const detailNode = screen.getByText(detail)

    expect(detailNode).toHaveClass("wrap-break-word")
    expect(detailNode).not.toHaveClass("line-clamp-2")
  })
})
