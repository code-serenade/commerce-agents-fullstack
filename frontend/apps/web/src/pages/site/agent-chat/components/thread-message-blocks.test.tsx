import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import type { ReactNode } from "react"
import { MemoryRouter } from "react-router"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import {
  AgentErrorActions,
  StudentNextActions,
  WelcomeCard,
} from "./thread-message-blocks"

describe("WelcomeCard", () => {
  it("lets students choose a task prompt to start", async () => {
    const user = userEvent.setup()
    const onSelectExamplePrompt = vi.fn()
    const i18n = await createAgentChatTestI18n()
    const prompt = i18n.t("agentChat.thread.studentTaskEntries.start.prompt")

    render(
      <WelcomeCardTestShell i18n={i18n}>
        <WelcomeCard
          content={i18n.t("agentChat.welcomeMessage")}
          onSelectExamplePrompt={onSelectExamplePrompt}
        />
      </WelcomeCardTestShell>
    )

    await user.click(screen.getByText(i18n.t("agentChat.thread.studentTaskEntries.start.title")))

    expect(onSelectExamplePrompt).toHaveBeenCalledWith(prompt)
  })

  it("shows task cards without long onboarding blocks", async () => {
    const i18n = await createAgentChatTestI18n()
    const examplePrompts = i18n.t("agentChat.thread.examplePrompts", {
      returnObjects: true,
    }) as string[]
    const capabilities = i18n.t("agentChat.thread.welcomeCapabilities", {
      returnObjects: true,
    }) as string[]

    render(
      <WelcomeCardTestShell i18n={i18n}>
        <WelcomeCard content={i18n.t("agentChat.welcomeMessage")} />
      </WelcomeCardTestShell>
    )

    expect(
      screen.getByText(i18n.t("agentChat.thread.studentTaskEntries.title"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.studentTaskEntries.data.title"))
    ).toBeInTheDocument()
    expect(
      screen.queryByText(i18n.t("agentChat.thread.studentFlow.title"))
    ).not.toBeInTheDocument()
    expect(
      screen.queryByText(i18n.t("agentChat.thread.processPromise"))
    ).not.toBeInTheDocument()
    expect(
      screen.queryByText(i18n.t("agentChat.thread.studentPromptRecipe.title"))
    ).not.toBeInTheDocument()
    expect(
      screen.queryByText(i18n.t("agentChat.thread.startHere"))
    ).not.toBeInTheDocument()
    expect(screen.queryByText(examplePrompts[0]!)).not.toBeInTheDocument()
    expect(screen.queryByText(capabilities[0]!)).not.toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.welcomeMessage"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.homeRoleCluster.title"))
    ).toBeInTheDocument()
    expect(
      document.querySelector("[data-agent-portrait-asset]")
    ).toBeInTheDocument()
  })

  it("lets students start from task-type entries", async () => {
    const user = userEvent.setup()
    const onSelectExamplePrompt = vi.fn()
    const i18n = await createAgentChatTestI18n()
    const prompt = i18n.t("agentChat.thread.studentTaskEntries.plan.prompt")

    render(
      <WelcomeCardTestShell i18n={i18n}>
        <WelcomeCard
          content={i18n.t("agentChat.welcomeMessage")}
          onSelectExamplePrompt={onSelectExamplePrompt}
        />
      </WelcomeCardTestShell>
    )

    expect(
      screen.getByText(i18n.t("agentChat.thread.studentTaskEntries.title"))
    ).toBeInTheDocument()

    await user.click(screen.getByText(i18n.t("agentChat.thread.studentTaskEntries.plan.title")))

    expect(onSelectExamplePrompt).toHaveBeenCalledWith(prompt)
  })

  it("lets students start from the data review entry", async () => {
    const user = userEvent.setup()
    const onSelectExamplePrompt = vi.fn()
    const i18n = await createAgentChatTestI18n()
    const prompt = i18n.t("agentChat.thread.studentTaskEntries.data.prompt")

    render(
      <WelcomeCardTestShell i18n={i18n}>
        <WelcomeCard
          content={i18n.t("agentChat.welcomeMessage")}
          onSelectExamplePrompt={onSelectExamplePrompt}
        />
      </WelcomeCardTestShell>
    )

    await user.click(screen.getByText(i18n.t("agentChat.thread.studentTaskEntries.data.title")))

    expect(onSelectExamplePrompt).toHaveBeenCalledWith(prompt)
  })
})

function WelcomeCardTestShell({
  children,
  i18n,
}: {
  children: ReactNode
  i18n: Parameters<typeof AgentChatTestI18nProvider>[0]["i18n"]
}) {
  return (
    <MemoryRouter>
      <AgentChatTestI18nProvider i18n={i18n}>
        {children}
      </AgentChatTestI18nProvider>
    </MemoryRouter>
  )
}

describe("AgentErrorActions", () => {
  it("turns reply errors into action buttons", async () => {
    const user = userEvent.setup()
    const onSelectPrompt = vi.fn()
    const i18n = await createAgentChatTestI18n()
    const retryPrompt = "Retry this product analysis"

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <AgentErrorActions
          fallbackPrompt={retryPrompt}
          onSelectPrompt={onSelectPrompt}
        />
      </AgentChatTestI18nProvider>
    )

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.errorActions.retry"),
      })
    )

    expect(onSelectPrompt).toHaveBeenCalledWith(retryPrompt)

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.errorActions.simplify"),
      })
    )

    expect(onSelectPrompt).toHaveBeenLastCalledWith(
      i18n.t("agentChat.thread.errorActions.shorterPrompt")
    )
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
