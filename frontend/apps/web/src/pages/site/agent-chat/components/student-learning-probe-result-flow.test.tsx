import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { MemoryRouter } from "react-router"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { StudentLearningProbe } from "./student-learning-probe"

const QUESTION = "Which traffic metric would you check first?"

describe("StudentLearningProbe result flow", () => {
  it("shows reviewed feedback and public sources in the original card", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()
    const onContinue = vi.fn()
    const sourceTitle = "Measure ecommerce"

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={vi.fn()}
            onEngage={vi.fn().mockResolvedValue(true)}
            onContinue={onContinue}
            onSkip={vi.fn()}
            probe={{
              engaged: true,
              id: "probe-result",
              questionText: QUESTION,
            }}
            ready
            result={{
              canContinue: true,
              demonstratedPoints: ["compare_funnel_stages"],
              feedbackCode: "learning_probe_correct",
              id: "probe-result",
              outcome: "correct",
              referenceExplanation: "Compare each stage of the funnel.",
              score: 1,
              selectedChoiceId: "compare_funnel",
              selectedChoiceText:
                "Compare the funnel stages before changing spend",
              correctChoiceText:
                "Compare the funnel stages before changing spend",
              answerComparisonCode: "matches_reference",
              knowledgePoints: [
                "Compare stage-by-stage conversion before changing spend",
              ],
              sourceLabels: [
                {
                  publisher: "Google Analytics",
                  sourceId: "google.analytics.ga4.ecommerce",
                  title: sourceTitle,
                  url: "https://developers.google.com/analytics/devguides/collection/ga4/ecommerce",
                },
              ],
              status: "answered",
            }}
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.outcomeStatus.correct")
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText("Compare each stage of the funnel.")
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.learningProbe.correctChoice"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        "Compare stage-by-stage conversion before changing spend"
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.assessmentImpactTitle")
      )
    ).toBeVisible()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.assessmentImpact.correct")
      )
    ).toBeVisible()
    expect(
      screen.getByText(
        i18n.t(
          "agentChat.thread.learningProbe.answerComparison.matches_reference"
        )
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.learningProbe.selectedChoice"))
    ).toBeInTheDocument()
    expect(
      screen.getAllByText("Compare the funnel stages before changing spend")
    ).toHaveLength(2)
    expect(
      screen.getByRole("link", { name: /Google Analytics/ })
    ).toHaveAttribute(
      "href",
      "https://developers.google.com/analytics/devguides/collection/ga4/ecommerce"
    )
    expect(
      screen.getByText(i18n.t("agentChat.thread.learningProbe.recorded"))
    ).toBeInTheDocument()
    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.continue"),
      })
    )
    expect(onContinue).toHaveBeenCalledWith("probe-result")
  })

  it("retains feedback and offers retry when the next question fails", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()
    const onContinue = vi.fn()
    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            continueFailed
            onAnswer={vi.fn()}
            onContinue={onContinue}
            onEngage={vi.fn().mockResolvedValue(true)}
            onSkip={vi.fn()}
            probe={{ engaged: true, id: "probe-retry", questionText: QUESTION }}
            ready
            result={{
              canContinue: true,
              demonstratedPoints: [],
              id: "probe-retry",
              outcome: "correct",
              sourceLabels: [],
              status: "answered",
            }}
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )
    expect(
      screen.getByText(i18n.t("agentChat.thread.learningProbe.continueFailed"))
    ).toBeVisible()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.outcomeStatus.correct")
      )
    ).toBeVisible()
    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.retryContinue"),
      })
    )
    expect(onContinue).toHaveBeenCalledTimes(1)
  })

  it("keeps feedback without a retry action when no further question exists", async () => {
    const i18n = await createAgentChatTestI18n()
    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            continueUnavailable
            onAnswer={vi.fn()}
            onContinue={vi.fn()}
            onEngage={vi.fn().mockResolvedValue(true)}
            onSkip={vi.fn()}
            probe={{
              engaged: true,
              id: "probe-no-more",
              questionText: QUESTION,
            }}
            ready
            result={{
              canContinue: true,
              demonstratedPoints: [],
              id: "probe-no-more",
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
        i18n.t("agentChat.thread.learningProbe.continueUnavailable")
      )
    ).toBeVisible()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.retryContinue"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen
        .getByText(
          i18n.t("agentChat.thread.learningProbe.outcomeStatus.correct")
        )
        .closest('[role="status"]')
    ).toHaveAttribute("aria-live", "polite")
  })

})
