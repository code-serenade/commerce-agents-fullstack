import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { MemoryRouter } from "react-router"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { StudentLearningProbe } from "./student-learning-probe"

const QUESTION = "Which traffic action is most useful next?"

describe("StudentLearningProbe choice resilience", () => {
  it("keeps an answered choice closed and retries feedback in place", async () => {
    const user = userEvent.setup()
    const i18n = await createAgentChatTestI18n()
    const onRetryResult = vi.fn()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={vi.fn()}
            onEngage={vi.fn().mockResolvedValue(true)}
            onRetryResult={onRetryResult}
            onSkip={vi.fn()}
            onSubmitChoice={vi.fn().mockResolvedValue(true)}
            probe={{
              choices: [{ id: "one", text: "Reviewed option" }],
              engaged: true,
              id: "probe-choice-result-failed",
              questionText: QUESTION,
              questionType: "single_choice",
            }}
            ready
            resultFailed
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )

    expect(screen.queryByRole("radio")).not.toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.learningProbe.resultFailed"))
    ).toBeVisible()
    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.retryResult"),
      })
    )
    expect(onRetryResult).toHaveBeenCalledTimes(1)
  })

  it("ignores postponing while a choice submission is already starting", async () => {
    const user = userEvent.setup()
    const i18n = await createAgentChatTestI18n()
    let resolveSubmission!: (value: boolean) => void
    const onSubmitChoice = vi.fn(
      () =>
        new Promise<boolean>((resolve) => {
          resolveSubmission = resolve
        })
    )
    const onSkip = vi.fn()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={vi.fn()}
            onEngage={vi.fn().mockResolvedValue(true)}
            onSubmitChoice={onSubmitChoice}
            onSkip={onSkip}
            probe={{
              choices: [{ id: "one", text: "Reviewed option" }],
              id: "probe-choice-submit-race",
              questionText: QUESTION,
              questionType: "single_choice",
            }}
            ready
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )

    await user.click(screen.getByRole("radio", { name: "Reviewed option" }))
    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.submitChoice"),
      })
    )
    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.later"),
      })
    )

    expect(onSubmitChoice).toHaveBeenCalledTimes(1)
    expect(onSkip).not.toHaveBeenCalled()
    resolveSubmission(true)
  })

  it("preserves the selected choice while collapsed", async () => {
    const user = userEvent.setup()
    const i18n = await createAgentChatTestI18n()
    const onSkip = vi.fn()
    const onSubmitChoice = vi.fn().mockResolvedValue(true)

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={vi.fn()}
            onEngage={vi.fn().mockResolvedValue(true)}
            onSubmitChoice={onSubmitChoice}
            onSkip={onSkip}
            probe={{
              choices: [{ id: "one", text: "Reviewed option" }],
              id: "probe-choice-skip-race",
              questionText: QUESTION,
              questionType: "single_choice",
            }}
            ready
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )

    await user.click(screen.getByRole("radio", { name: "Reviewed option" }))
    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.later"),
      })
    )
    expect(onSkip).not.toHaveBeenCalled()
    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.expand"),
      })
    )
    expect(screen.getByRole("radio", { name: "Reviewed option" })).toBeChecked()
    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.submitChoice"),
      })
    )

    expect(onSubmitChoice).toHaveBeenCalledTimes(1)
  })
})
