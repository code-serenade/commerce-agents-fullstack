import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { MessageContextSummary } from "./message-context-summary"

const readableFileLabel = "readable.pdf"
const pendingFileLabel = "pending.docx"

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

  it("shows source readiness from the sent turn context summary", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MessageContextSummary
          summary={{
            campaignIds: [],
            campaigns: [],
            productIds: [],
            products: [],
            selectedRoles: [],
            sourceRefs: ["conversation_file:file-readable", "conversation_file:file-pending"],
            sources: [
              {
                id: "conversation_file:file-readable",
                label: readableFileLabel,
                readiness: "readable",
              },
              {
                id: "conversation_file:file-pending",
                label: pendingFileLabel,
                readiness: "pending",
              },
            ],
          }}
        />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(
        i18n.t("agentChat.thread.contextSummary.sourceCountWithReadiness", {
          count: 2,
          readable: 1,
          unreadable: 1,
        })
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.contextSummary.sourceReadiness.readable")
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.contextSummary.sourceReadiness.pending")
      )
    ).toBeInTheDocument()
  })
})
