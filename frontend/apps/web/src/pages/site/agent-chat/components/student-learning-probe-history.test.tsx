import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { createProbe, createResult } from "../view-model/use-student-learning-probe-test-utils"
import { StudentLearningProbeHistory } from "./student-learning-probe-history"

describe("StudentLearningProbeHistory", () => {
  it("keeps earlier questions visible and expands their complete feedback", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()
    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <StudentLearningProbeHistory
          items={[
            {
              probe: createProbe({ questionText: "How should the title show size?" }),
              result: createResult({
                correctChoiceText: "Describe the same product and its size",
                referenceExplanation: "A clear title distinguishes the product variant.",
              }),
            },
            {
              probe: createProbe({
                id: "incorrect-probe",
                questionText: "Should a title contain a promotion deadline?",
              }),
              result: createResult({
                id: "incorrect-probe",
                outcome: "incorrect",
              }),
            },
          ]}
        />
      </AgentChatTestI18nProvider>
    )

    expect(screen.getByText("How should the title show size?")).toBeVisible()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.outcomeStatus.correct")
      )
    ).toBeVisible()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.outcomeStatus.incorrect")
      )
    ).toBeVisible()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.outcomeStatus.incorrect")
      )
        .closest("[data-learning-probe-outcome]")
    ).toHaveAttribute("data-learning-probe-outcome", "incorrect")
    expect(
      screen.getByText("A clear title distinguishes the product variant.")
    ).not.toBeVisible()

    await user.click(
      screen.getByText(i18n.t("agentChat.thread.learningProbe.historyItem", { number: 1 }))
    )

    expect(
      screen.getByText("A clear title distinguishes the product variant.")
    ).toBeVisible()
    expect(screen.getByText("Describe the same product and its size")).toBeVisible()
  })
})
