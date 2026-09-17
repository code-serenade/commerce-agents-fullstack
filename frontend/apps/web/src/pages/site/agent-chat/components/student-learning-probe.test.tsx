import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { MemoryRouter } from "react-router"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { StudentLearningProbe } from "./student-learning-probe"

const CONTEXTUAL_QUESTION = "Which traffic metric would you check first?"
const JUDGMENT_QUESTION = "What is your judgment?"
describe("StudentLearningProbe", () => {
  it("reveals a probe that appears while the thread is near the bottom", async () => {
    const i18n = await createAgentChatTestI18n()
    const scrollRoot = document.createElement("section")
    const scrollTo = vi.fn()
    scrollRoot.dataset.agentChatThreadScrollRoot = "true"
    Object.defineProperties(scrollRoot, {
      clientHeight: { configurable: true, value: 600 },
      scrollHeight: { configurable: true, value: 900 },
      scrollTop: { configurable: true, value: 180 },
      scrollTo: { configurable: true, value: scrollTo },
    })
    document.body.append(scrollRoot)

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={vi.fn()}
            onEngage={vi.fn().mockResolvedValue(true)}
            onSkip={vi.fn()}
            probe={{ id: "probe-visible", questionText: JUDGMENT_QUESTION }}
            ready
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>,
      { container: scrollRoot }
    )

    await waitFor(() =>
      expect(scrollTo).toHaveBeenCalledWith({
        behavior: "smooth",
        top: 900,
      })
    )
    scrollRoot.remove()
  })

  it("explains the benefit and engages before opening the existing composer", async () => {
    const user = userEvent.setup()
    const i18n = await createAgentChatTestI18n()
    const onAnswer = vi.fn()
    const onEngage = vi.fn().mockResolvedValue(true)

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={onAnswer}
            onEngage={onEngage}
            onSkip={vi.fn()}
            probe={{
              id: "probe-a",
              interactionMode: "contextual_micro",
              questionText: CONTEXTUAL_QUESTION,
              rewardCode: "progress_feedback",
            }}
            ready
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.reward.progress_feedback")
      )
    ).toBeInTheDocument()

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.answer"),
      })
    )

    await waitFor(() => expect(onEngage).toHaveBeenCalledWith("probe-a"))
    expect(onAnswer).toHaveBeenCalledWith(
      i18n.t("agentChat.thread.learningProbe.answerStarter")
    )
    expect(screen.getByText(CONTEXTUAL_QUESTION)).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.learningProbe.waitingAnswer"))
    ).toBeInTheDocument()
    expect(
      screen.queryByText(
        i18n.t("agentChat.thread.learningProbe.selectedChoice")
      )
    ).not.toBeInTheDocument()
  })

  it("collapses without skipping and can be expanded again", async () => {
    const user = userEvent.setup()
    const i18n = await createAgentChatTestI18n()
    const onSkip = vi.fn()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={vi.fn()}
            onEngage={vi.fn().mockResolvedValue(true)}
            onSkip={onSkip}
            probe={{ id: "probe-b", questionText: JUDGMENT_QUESTION }}
            ready
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.later"),
      })
    )
    expect(onSkip).not.toHaveBeenCalled()
    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.expand"),
      })
    ).toBeVisible()
    await user.click(screen.getByText(JUDGMENT_QUESTION))
    expect(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.answer"),
      })
    ).toBeVisible()
  })

  it("keeps the prompt available when engagement fails", async () => {
    const user = userEvent.setup()
    const i18n = await createAgentChatTestI18n()
    const onAnswer = vi.fn()
    const onEngage = vi.fn().mockResolvedValue(false)

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter>
          <StudentLearningProbe
            onAnswer={onAnswer}
            onEngage={onEngage}
            onSkip={vi.fn()}
            probe={{ id: "probe-c", questionText: CONTEXTUAL_QUESTION }}
            ready
          />
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.answer"),
      })
    )

    await waitFor(() => expect(onEngage).toHaveBeenCalledWith("probe-c"))
    expect(onAnswer).not.toHaveBeenCalled()
    expect(screen.getByText(CONTEXTUAL_QUESTION)).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.learningProbe.engageFailed"))
    ).toBeVisible()
  })
})
