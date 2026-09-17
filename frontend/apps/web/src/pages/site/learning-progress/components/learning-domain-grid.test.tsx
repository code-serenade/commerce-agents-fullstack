import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import type { LearningDomainAssessment } from "@/api"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { LearningDomainGrid } from "./learning-domain-grid"

const assessment: LearningDomainAssessment = {
  knowledgeCode: "conversion",
  statusCode: "needs_attention",
  reasonCode: "latest_answer_incorrect",
  observed: true,
  observationCount: 2,
  assessmentCount: 3,
  distinctQuestionCount: 2,
  assessmentAngleCount: 2,
  correctCount: 1,
  partialCount: 1,
  incorrectCount: 1,
  latestOutcome: "incorrect",
  latestAssessedAt: "2026-09-08T08:00:00Z",
  retentionVerified: false,
  missingEvidenceCodes: ["positive_correction"],
  nextActionCode: "correct_latest_gap",
  recentEvidence: [
    {
      evidenceId: "evidence-1",
      conversationId: "conversation-1",
      questionId: "conversion.question.001",
      questionText: "Which event starts checkout?",
      selectedChoiceText: "purchase",
      outcome: "incorrect",
      createdAt: "2026-09-08T08:00:00Z",
    },
  ],
}

describe("LearningDomainGrid", () => {
  it("shows the reason, evidence gap, next action, and traceable answer record", async () => {
    const i18n = await createAgentChatTestI18n()
    const onOpenConversation = vi.fn()
    const user = userEvent.setup()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <LearningDomainGrid
          domains={[
            {
              assessment,
              code: "conversion",
              evaluationStatus: "needs_attention",
              label: i18n.t("learningProgress.domains.conversion"),
              observed: true,
            },
          ]}
          onOpenConversation={onOpenConversation}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(
        i18n.t("learningProgress.domainReasons.latest_answer_incorrect")
      )
    ).toBeVisible()
    expect(
      screen.getByText(i18n.t("learningProgress.domainRuleCheck.title"))
    ).toBeVisible()
    expect(
      screen.getByText(
        i18n.t(
          "learningProgress.domainRuleCheck.conditions.positive_multi_angle.title"
        )
      )
    ).toBeVisible()
    expect(
      screen.getAllByText(i18n.t("learningProgress.domainRuleCheck.notMet"))
    ).toHaveLength(2)
    expect(
      screen.getByText(
        i18n.t("learningProgress.domainMissingEvidence.positive_correction")
      )
    ).toBeVisible()
    expect(
      screen.getByText(
        i18n.t("learningProgress.domainNextActions.correct_latest_gap")
      )
    ).toBeVisible()
    expect(screen.getByText("Which event starts checkout?")).toBeVisible()
    expect(screen.getByText(/purchase/)).toBeVisible()

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("learningProgress.viewRelatedConversation"),
      })
    )
    expect(onOpenConversation).toHaveBeenCalledWith("conversation-1")
  })
})
