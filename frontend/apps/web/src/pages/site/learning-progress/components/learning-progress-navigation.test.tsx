import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { LearningProgressNavigation } from "./learning-progress-navigation"

describe("LearningProgressNavigation", () => {
  it("switches learning views without navigating away", async () => {
    const i18n = await createAgentChatTestI18n()
    const onViewChange = vi.fn()
    const user = userEvent.setup()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <LearningProgressNavigation
          activeView="overview"
          onViewChange={onViewChange}
        />
      </AgentChatTestI18nProvider>
    )

    await user.click(
      screen.getByRole("tab", {
        name: i18n.t("learningProgress.views.portfolio"),
      })
    )

    expect(onViewChange).toHaveBeenCalledWith("portfolio")
    expect(screen.getAllByRole("tab")).toHaveLength(4)
    expect(
      screen.getByRole("tab", {
        name: i18n.t("learningProgress.views.map"),
      })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("tab", {
        name: i18n.t("learningProgress.views.portfolio"),
      })
    ).toBeInTheDocument()
  })
})
