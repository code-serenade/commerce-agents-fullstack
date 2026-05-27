import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { AgentChatSourceContextBar } from "./agent-chat-source-context-bar"

const readableFileTitle = "readable.pdf"
const pendingFileTitle = "pending.docx"
const failedFileTitle = "failed.txt"
const unsupportedFileTitle = "unsupported.xls"

describe("AgentChatSourceContextBar", () => {
  it("exposes selected material read status in the compact button label", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <AgentChatSourceContextBar
          availableSources={[]}
          disabled={false}
          onClearSourceContext={vi.fn()}
          onToggleSourceContext={vi.fn()}
          selectedSources={[
            {
              id: "file-readable",
              kind: "conversation_file",
              readable: true,
              ref: "conversation_file:file-readable",
              status: "uploaded",
              title: readableFileTitle,
            },
            {
              id: "file-pending",
              kind: "conversation_file",
              readable: false,
              ref: "conversation_file:file-pending",
              status: "pending",
              title: pendingFileTitle,
            },
            {
              id: "file-failed",
              kind: "conversation_file",
              readable: false,
              ref: "conversation_file:file-failed",
              status: "failed",
              title: failedFileTitle,
            },
            {
              id: "file-unsupported",
              kind: "conversation_file",
              readable: false,
              ref: "conversation_file:file-unsupported",
              status: "unsupported",
              title: unsupportedFileTitle,
            },
          ]}
        />
      </AgentChatTestI18nProvider>
    )

    const button = screen.getByRole("button", {
      name: new RegExp(i18n.t("agentChat.composer.sourceContext.compactTitle")),
    })

    expect(button).toHaveAccessibleName(
      expect.stringContaining(
        i18n.t("agentChat.composer.sourceContext.selectedReadiness.readable", {
          count: 1,
        })
      )
    )
    expect(button).toHaveAccessibleName(
      expect.stringContaining(
        i18n.t("agentChat.composer.sourceContext.selectedReadiness.pending", {
          count: 1,
        })
      )
    )
    expect(button).toHaveAccessibleName(
      expect.stringContaining(
        i18n.t("agentChat.composer.sourceContext.selectedReadiness.failed", {
          count: 1,
        })
      )
    )
    expect(button).toHaveAccessibleName(
      expect.stringContaining(
        i18n.t(
          "agentChat.composer.sourceContext.selectedReadiness.unsupported",
          { count: 1 }
        )
      )
    )
    expect(screen.getByText("3")).toBeInTheDocument()
  })
})
