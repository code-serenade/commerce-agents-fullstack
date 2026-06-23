import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import zhMessages from "@/../lang/zhCN/messages"
import { MessageContextSummary } from "./message-context-summary"

const readableFileLabel = "readable.pdf"
const pendingFileLabel = "pending.docx"
const readableSourceSummary = textFromCodes(
  50,
  32,
  20221,
  36164,
  26009,
  21644,
  22270,
  29255,
  65292,
  49,
  32,
  20221,
  21487,
  35835
)
const rawSourceReadinessKey =
  "agentChat.thread.contextSummary.sourceCountWithReadiness"
const answerStylePrefix = textFromCodes(22238, 31572, 26041, 24335, 65306)
const oldModePrefix = textFromCodes(27169, 24335, 65306)

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

  it("keeps Chinese source readiness summary user-facing instead of showing locale keys", () => {
    const contextSummary = zhMessages.agentChat.thread.contextSummary

    expect(
      contextSummary.sourceCountWithReadiness
        .replace("{{count}}", "2")
        .replace("{{readable}}", "1")
    ).toBe(readableSourceSummary)
    expect(contextSummary.sourceCountWithReadiness).not.toBe(rawSourceReadinessKey)
  })

  it("labels response preference as answer style instead of a vague mode", () => {
    const contextSummary = zhMessages.agentChat.thread.contextSummary

    expect(
      contextSummary.strategy.replace("{{strategy}}", "")
    ).toBe(answerStylePrefix)
    expect(contextSummary.strategy).not.toContain(oldModePrefix)
  })
})

function textFromCodes(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
