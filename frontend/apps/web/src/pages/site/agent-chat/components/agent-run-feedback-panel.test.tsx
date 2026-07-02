import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { AgentRunFeedbackPanel } from "./agent-run-feedback-panel"

describe("AgentRunFeedbackPanel", () => {
  it("shows feedback target state without exposing the internal run id", async () => {
    const i18n = await createAgentChatTestI18n()
    const runId = "12345678-90ab-cdef-1234-567890abcdef"

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <AgentRunFeedbackPanel
          feedback={{
            canSubmit: true,
            comment: "",
            disabledReason: null,
            error: null,
            isSubmitting: false,
            rating: 5,
            setComment: vi.fn(),
            setRating: vi.fn(),
            submitted: false,
            submit: vi.fn(async () => {}),
            target: {
              runId,
              runStatus: "completed",
              workspaceId: "workspace-1",
              workspaceTitle: "Sunscreen jacket check",
            },
          }}
        />
      </AgentChatTestI18nProvider>
    )

    expect(screen.getByText(i18n.t("agentChat.feedback.linked"))).toBeInTheDocument()
    expect(screen.queryByText(runId.slice(0, 8))).not.toBeInTheDocument()
  })
})
