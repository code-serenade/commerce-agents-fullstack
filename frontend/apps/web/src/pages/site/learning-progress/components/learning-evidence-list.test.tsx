import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import type { LearningEvidenceEntry } from "@/api"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { LearningEvidenceList } from "./learning-evidence-list"

const baseEvidence: LearningEvidenceEntry = {
  id: "evidence-1",
  knowledgeCode: "conversion",
  evidenceType: "probe_answer",
  direction: "positive",
  strength: 1,
  summary: "Question: the key event in a checkout flow",
  assessmentOutcome: "correct",
  questionId: "conversion.checkout_event.001",
  rubricVersion: 1,
  createdAt: "2026-08-27T08:00:00Z",
}

describe("LearningEvidenceList conversation traceability", () => {
  it("shows the question and selected choice as structured fields", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <LearningEvidenceList
          evidence={[
            {
              ...baseEvidence,
              questionText: "Which event starts the checkout flow?",
              selectedChoiceId: "begin_checkout",
              selectedChoiceText: "begin_checkout",
            },
          ]}
          hasError={false}
          isLoading={false}
          onOpenConversation={vi.fn()}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(i18n.t("learningProgress.evidenceQuestion"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("learningProgress.evidenceSelectedChoice"))
    ).toBeInTheDocument()
    expect(
      screen.getByText("Which event starts the checkout flow?")
    ).toBeInTheDocument()
    expect(screen.getByText("begin_checkout")).toBeInTheDocument()
    expect(screen.queryByText(baseEvidence.summary)).not.toBeInTheDocument()
  })

  it("opens the conversation associated with a learning record", async () => {
    const user = userEvent.setup()
    const i18n = await createAgentChatTestI18n()
    const onOpenConversation = vi.fn()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <LearningEvidenceList
          evidence={[
            { ...baseEvidence, conversationId: "conversation-source" },
          ]}
          hasError={false}
          isLoading={false}
          onOpenConversation={onOpenConversation}
        />
      </AgentChatTestI18nProvider>
    )

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("learningProgress.viewRelatedConversation"),
      })
    )

    expect(onOpenConversation).toHaveBeenCalledOnce()
    expect(onOpenConversation).toHaveBeenCalledWith("conversation-source")
  })

  it("keeps legacy records without a conversation action", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <LearningEvidenceList
          evidence={[baseEvidence]}
          hasError={false}
          isLoading={false}
          onOpenConversation={vi.fn()}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.queryByRole("button", {
        name: i18n.t("learningProgress.viewRelatedConversation"),
      })
    ).not.toBeInTheDocument()
    expect(screen.getByText(baseEvidence.summary)).toBeInTheDocument()
  })
})
