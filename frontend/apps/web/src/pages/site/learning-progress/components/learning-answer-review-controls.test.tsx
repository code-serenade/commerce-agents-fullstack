import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { LearningAnswerReviewControls } from "./learning-answer-review-controls"

describe("LearningAnswerReviewControls", () => {
  it("switches outcome and time filters independently", async () => {
    const i18n = await createAgentChatTestI18n()
    const onReviewOnlyChange = vi.fn()
    const onEvidenceRangeChange = vi.fn()
    const user = userEvent.setup()
    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <LearningAnswerReviewControls
          evidenceRange="recent"
          onEvidenceRangeChange={onEvidenceRangeChange}
          onReviewOnlyChange={onReviewOnlyChange}
          reviewOnly={false}
        />
      </AgentChatTestI18nProvider>
    )
    await user.click(screen.getByRole("tab", {
      name: i18n.t("learningProgress.portfolio.reviewAnswers"),
    }))
    expect(onReviewOnlyChange).toHaveBeenCalledWith(true)
    expect(onEvidenceRangeChange).not.toHaveBeenCalled()
    await user.click(screen.getByRole("tab", {
      name: i18n.t("learningProgress.evidenceRanges.days90"),
    }))
    expect(onEvidenceRangeChange).toHaveBeenCalledWith("90d")
  })
})
