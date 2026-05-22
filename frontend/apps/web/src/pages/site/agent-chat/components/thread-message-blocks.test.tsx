import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { StudentNextActions, WelcomeCard } from "./thread-message-blocks"

describe("WelcomeCard", () => {
  it("lets students choose an example prompt to start", async () => {
    const user = userEvent.setup()
    const onSelectExamplePrompt = vi.fn()
    const i18n = await createAgentChatTestI18n()
    const prompts = i18n.t("agentChat.thread.examplePrompts", {
      returnObjects: true,
    }) as string[]
    const prompt = prompts[0]!

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <WelcomeCard
          content={i18n.t("agentChat.thread.agentName")}
          onSelectExamplePrompt={onSelectExamplePrompt}
        />
      </AgentChatTestI18nProvider>
    )

    await user.click(screen.getByText(prompt))

    expect(onSelectExamplePrompt).toHaveBeenCalledWith(prompt)
  })

  it("shows a simple student workflow in the welcome card", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <WelcomeCard content={i18n.t("agentChat.thread.agentName")} />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(i18n.t("agentChat.thread.studentFlow.ask.title"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.studentFlow.watch.title"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.studentFlow.deliver.title"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.studentFlow.title"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.processPromise"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.studentPromptRecipe.title"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.studentPromptRecipe.goal.label"))
    ).toBeInTheDocument()
  })

  it("lets students start from task-type entries", async () => {
    const user = userEvent.setup()
    const onSelectExamplePrompt = vi.fn()
    const i18n = await createAgentChatTestI18n()
    const prompt = i18n.t("agentChat.thread.studentTaskEntries.plan.prompt")

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <WelcomeCard
          content={i18n.t("agentChat.thread.agentName")}
          onSelectExamplePrompt={onSelectExamplePrompt}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(i18n.t("agentChat.thread.studentTaskEntries.title"))
    ).toBeInTheDocument()

    await user.click(screen.getByText(prompt))

    expect(onSelectExamplePrompt).toHaveBeenCalledWith(prompt)
  })
})

describe("StudentNextActions", () => {
  it("lets students continue from suggested next steps", async () => {
    const user = userEvent.setup()
    const onSelectPrompt = vi.fn()
    const i18n = await createAgentChatTestI18n()
    const prompts = i18n.t("agentChat.thread.nextActions.groups.default", {
      returnObjects: true,
    }) as string[]
    const prompt = prompts[0]!

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <StudentNextActions onSelectPrompt={onSelectPrompt} />
      </AgentChatTestI18nProvider>
    )

    await user.click(screen.getByText(prompt))

    expect(onSelectPrompt).toHaveBeenCalledWith(prompt)
  })

  it("changes suggested next steps from the assistant answer context", async () => {
    const i18n = await createAgentChatTestI18n()
    const productPrompts = i18n.t(
      "agentChat.thread.nextActions.groups.product",
      {
        returnObjects: true,
      }
    ) as string[]
    const dataPrompts = i18n.t("agentChat.thread.nextActions.groups.data", {
      returnObjects: true,
    }) as string[]

    const { rerender } = render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <StudentNextActions
          message={{
            id: "assistant-product",
            role: "assistant",
            content: "product risk and audience",
            contextSummary: {
              campaignIds: [],
              campaigns: [],
              productIds: ["product-1"],
              products: [],
              selectedRoles: ["product"],
              sourceRefs: [],
              sources: [],
            },
          }}
        />
      </AgentChatTestI18nProvider>
    )

    expect(screen.getByText(productPrompts[0]!)).toBeInTheDocument()

    rerender(
      <AgentChatTestI18nProvider i18n={i18n}>
        <StudentNextActions
          message={{
            id: "assistant-data",
            role: "assistant",
            content: "data diagnosis and conversion metric",
          }}
        />
      </AgentChatTestI18nProvider>
    )

    expect(screen.getByText(dataPrompts[0]!)).toBeInTheDocument()
  })
})
