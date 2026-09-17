import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { LearningEvidenceHistory } from "./learning-evidence-history"

describe("LearningEvidenceHistory", () => {
  it("separates structured assessments from conversation observations", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <LearningEvidenceHistory
          evidence={[]}
          evidenceError={false}
          evidenceLoading={false}
          evidenceRange="recent"
          onEvidenceRangeChange={vi.fn()}
          onOpenConversation={vi.fn()}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByLabelText(i18n.t("learningProgress.evidenceLegendLabel"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("learningProgress.evidenceLegend.assessedMeaning"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("learningProgress.evidenceLegend.observedMeaning"))
    ).toBeInTheDocument()
  })
})
