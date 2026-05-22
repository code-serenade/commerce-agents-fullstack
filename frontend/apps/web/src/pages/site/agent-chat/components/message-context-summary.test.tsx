import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { MessageContextSummary } from "./message-context-summary"

describe("MessageContextSummary", () => {
  it("describes auto role assignment without showing zero selected roles", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MessageContextSummary
          summary={{
            campaignIds: [],
            campaigns: [],
            collaborationMode: "auto",
            primaryRole: "ops",
            productIds: [],
            products: [],
            selectedRoles: [],
            sourceRefs: [],
            sources: [],
          }}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(i18n.t("agentChat.thread.contextSummary.autoRoles"))
    ).toBeInTheDocument()
    expect(
      screen.queryByText(
        i18n.t("agentChat.thread.contextSummary.roles", {
          count: 0,
          role: "ops",
        })
      )
    ).not.toBeInTheDocument()
  })
})
