import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import type { LearningEvidenceEntry } from "@/api"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { LearningReviewCenter } from "./learning-review-center"

describe("LearningReviewCenter", () => {
  it("shows a structured review record and returns to its conversation", async () => {
    const i18n = await createAgentChatTestI18n()
    const onOpenConversation = vi.fn()
    const user = userEvent.setup()
    const evidence: LearningEvidenceEntry[] = [
      {
        id: "evidence-1",
        conversationId: "conversation-1",
        questionText: "Which claim is verifiable?",
        selectedChoiceText: "Best seller everywhere",
        knowledgeCode: "content",
        evidenceType: "probe_answer",
        direction: "negative",
        strength: 1,
        summary: "Incorrect answer",
        assessmentOutcome: "incorrect",
        createdAt: "2026-09-16T08:00:00Z",
      },
    ]

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <LearningReviewCenter
          assessments={[]}
          evidence={evidence}
          hasError={false}
          isLoading={false}
          onContinueChat={vi.fn()}
          onOpenConversation={onOpenConversation}
        />
      </AgentChatTestI18nProvider>
    )

    expect(screen.getByText("Which claim is verifiable?")).toBeVisible()
    expect(screen.getByText("Best seller everywhere", { exact: false })).toBeVisible()

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("learningProgress.reviewCenter.openConversation"),
      })
    )
    expect(onOpenConversation).toHaveBeenCalledWith("conversation-1")
  })
})

