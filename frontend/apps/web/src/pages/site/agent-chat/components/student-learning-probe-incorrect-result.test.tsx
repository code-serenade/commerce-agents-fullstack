import { render, screen } from "@testing-library/react"
import { MemoryRouter } from "react-router"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { StudentLearningProbe } from "./student-learning-probe"

describe("StudentLearningProbe incorrect result", () => {
  it("distinguishes review guidance without relying on color", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={vi.fn()}
            onEngage={vi.fn().mockResolvedValue(true)}
            onSkip={vi.fn()}
            probe={{
              engaged: true,
              id: "probe-incorrect-impact",
              questionText: "Which traffic metric would you check first?",
            }}
            ready
            result={{
              answerComparisonCode: "differs_from_reference",
              correctChoiceText: "Compare the complete funnel",
              demonstratedPoints: [],
              id: "probe-incorrect-impact",
              knowledgePoints: ["Compare every decision condition"],
              outcome: "incorrect",
              referenceExplanation:
                "A complete comparison must include every relevant stage.",
              selectedChoiceText: "Look at clicks only",
              sourceLabels: [],
              status: "answered",
            }}
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.outcomeStatus.incorrect")
      )
    ).toBeVisible()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.outcomeDetail.incorrect")
      )
    ).toBeVisible()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.outcomeStatus.incorrect")
      ).closest("[data-learning-probe-outcome]")
    ).toHaveAttribute("data-learning-probe-outcome", "incorrect")
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.assessmentImpact.incorrect")
      )
    ).toBeVisible()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.reviewGuidance.incorrect")
      )
    ).toBeVisible()
  })
})
