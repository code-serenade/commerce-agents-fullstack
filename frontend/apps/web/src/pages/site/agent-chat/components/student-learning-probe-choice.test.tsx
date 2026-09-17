import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { MemoryRouter } from "react-router"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { StudentLearningProbe } from "./student-learning-probe"

const QUESTION = "Which traffic action is most useful next?"

describe("StudentLearningProbe choices", () => {
  it("submits one reviewed choice without using the existing composer", async () => {
    const user = userEvent.setup()
    const i18n = await createAgentChatTestI18n()
    const onAnswer = vi.fn()
    const onEngage = vi.fn().mockResolvedValue(true)
    const onSubmitChoice = vi.fn().mockResolvedValue(true)
    const correctChoice = "Compare the funnel stages before changing spend"

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={onAnswer}
            onEngage={onEngage}
            onSubmitChoice={onSubmitChoice}
            onSkip={vi.fn()}
            probe={{
              choices: [
                { id: "compare_funnel", text: correctChoice },
                { id: "increase_clicks", text: "Only increase clicks" },
              ],
              id: "probe-choice",
              questionText: QUESTION,
              questionType: "single_choice",
            }}
            ready
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.answer"),
      })
    ).not.toBeInTheDocument()
    expect(
      screen.queryByText(i18n.t("agentChat.thread.learningProbe.contextHint"))
    ).not.toBeInTheDocument()
    expect(
      screen.queryByText(i18n.t("agentChat.thread.learningProbe.choiceHint"))
    ).not.toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.learningProbe.questionLabel"))
    ).toBeVisible()
    expect(
      screen.queryByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.viewProgress"),
      })
    ).not.toBeInTheDocument()
    const question = screen.getByText(QUESTION)
    expect(question).toHaveAttribute(
      "id",
      "learning-probe-question-probe-choice"
    )
    expect(screen.getByRole("radiogroup")).toHaveAttribute(
      "aria-labelledby",
      question.id
    )
    await user.click(screen.getByRole("radio", { name: correctChoice }))

    expect(onEngage).not.toHaveBeenCalled()
    expect(onAnswer).not.toHaveBeenCalled()
    expect(screen.getByRole("radio", { name: correctChoice })).toBeChecked()
    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.submitChoice"),
      })
    )
    await waitFor(() =>
      expect(onSubmitChoice).toHaveBeenCalledWith(
        "probe-choice",
        "compare_funnel"
      )
    )
  })

  it("keeps the selected choice when structured submission fails", async () => {
    const user = userEvent.setup()
    const i18n = await createAgentChatTestI18n()
    const onAnswer = vi.fn()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={onAnswer}
            onEngage={vi.fn().mockResolvedValue(false)}
            onSubmitChoice={vi.fn().mockResolvedValue(false)}
            onSkip={vi.fn()}
            probe={{
              choices: [{ id: "one", text: "Reviewed option" }],
              id: "probe-choice-failed",
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
    await waitFor(() => expect(onAnswer).not.toHaveBeenCalled())
    expect(screen.getByRole("radio", { name: "Reviewed option" })).toBeChecked()
    expect(screen.getByText(QUESTION)).toBeInTheDocument()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.submitChoiceFailed")
      )
    ).toBeVisible()
  })

  it("keeps the full question visible while a selected answer is submitting", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={vi.fn()}
            onEngage={vi.fn().mockResolvedValue(true)}
            onSubmitChoice={vi.fn().mockResolvedValue(true)}
            onSkip={vi.fn()}
            probe={{
              choices: [{ id: "one", text: "Reviewed option" }],
              id: "probe-choice-submitting",
              questionText: QUESTION,
              questionType: "single_choice",
            }}
            ready
            submittingChoice
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )

    expect(screen.getByText(QUESTION)).toBeVisible()
    expect(
      screen.getByRole("radio", { name: "Reviewed option" })
    ).toBeDisabled()
    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.submittingChoice"),
      })
    ).toBeDisabled()
    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.later"),
      })
    ).toBeDisabled()
  })

  it("supports native keyboard selection without entering either answer flow", async () => {
    const user = userEvent.setup()
    const i18n = await createAgentChatTestI18n()
    const onAnswer = vi.fn()
    const onEngage = vi.fn().mockResolvedValue(true)
    const onSubmitChoice = vi.fn().mockResolvedValue(true)

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={onAnswer}
            onEngage={onEngage}
            onSubmitChoice={onSubmitChoice}
            onSkip={vi.fn()}
            probe={{
              choices: [
                { id: "first", text: "First reviewed option" },
                { id: "second", text: "Second reviewed option" },
                { id: "third", text: "Third reviewed option" },
              ],
              id: "probe-keyboard-choice",
              questionText: QUESTION,
              questionType: "single_choice",
            }}
            ready
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )

    const first = screen.getByRole("radio", {
      name: "First reviewed option",
    })
    const second = screen.getByRole("radio", {
      name: "Second reviewed option",
    })

    await user.tab()
    await user.tab()
    expect(first).toHaveFocus()
    await user.keyboard("[Space]")
    expect(first).toBeChecked()

    await user.keyboard("[ArrowDown]")
    expect(second).toHaveFocus()
    expect(second).toBeChecked()
    expect(onEngage).not.toHaveBeenCalled()
    expect(onAnswer).not.toHaveBeenCalled()
    expect(onSubmitChoice).not.toHaveBeenCalled()
  })

  it("shows evaluation feedback instead of re-enabling an answered choice", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={vi.fn()}
            onEngage={vi.fn().mockResolvedValue(true)}
            onSubmitChoice={vi.fn().mockResolvedValue(true)}
            onSkip={vi.fn()}
            probe={{
              choices: [{ id: "one", text: "Reviewed option" }],
              engaged: true,
              id: "probe-choice-answered",
              questionText: QUESTION,
              questionType: "single_choice",
            }}
            ready
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )

    expect(screen.queryByRole("radio")).not.toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.learningProbe.evaluating"))
    ).toBeVisible()
  })
})
