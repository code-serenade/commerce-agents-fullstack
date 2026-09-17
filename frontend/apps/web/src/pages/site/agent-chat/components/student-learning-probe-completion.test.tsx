import { render, screen } from "@testing-library/react"
import { MemoryRouter } from "react-router"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { StudentLearningProbe } from "./student-learning-probe"

describe("StudentLearningProbe completion", () => {
  it("explains when the relevant questions for this turn are complete", async () => {
    const i18n = await createAgentChatTestI18n()
    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={vi.fn()}
            onContinue={vi.fn()}
            onEngage={vi.fn().mockResolvedValue(true)}
            onSkip={vi.fn()}
            probe={{
              engaged: true,
              id: "probe-complete",
              questionText: "Which title is clearest?",
            }}
            ready
            result={{
              canContinue: false,
              demonstratedPoints: [],
              id: "probe-complete",
              outcome: "correct",
              sourceLabels: [],
              status: "answered",
            }}
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.continuationComplete")
      )
    ).toBeVisible()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.continue"),
      })
    ).not.toBeInTheDocument()
  })
})
