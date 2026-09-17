import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { AgentChatStatus } from "./agent-chat-status"

describe("AgentChatStatus", () => {
  it("requests login without forwarding the click event as a redirect target", async () => {
    const i18n = await createAgentChatTestI18n()
    const onLogin = vi.fn()
    const user = userEvent.setup()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <AgentChatStatus error={null} loginRequired onLogin={onLogin} />
      </AgentChatTestI18nProvider>
    )

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.loginRequired.action"),
      })
    )

    expect(onLogin).toHaveBeenCalledOnce()
    expect(onLogin).toHaveBeenCalledWith()
  })
})
