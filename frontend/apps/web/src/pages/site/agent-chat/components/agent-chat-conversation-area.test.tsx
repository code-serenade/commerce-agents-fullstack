import { fireEvent, render } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import type { AgentChatContextValue } from "../context/agent-chat-context-value"
import { AgentChatConversationArea } from "./agent-chat-conversation-area"

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}))

vi.mock("./agent-chat-composer", () => ({
  AgentChatComposer: () => <div data-testid="composer" />,
}))
vi.mock("./agent-chat-quota-status", () => ({
  AgentChatQuotaStatus: () => null,
}))
vi.mock("./agent-chat-report-actions", () => ({
  AgentChatReportActions: () => null,
}))
vi.mock("./agent-chat-thread", () => ({
  AgentChatThread: () => <div data-testid="thread" />,
}))
vi.mock("./agent-chat-welcome-content", () => ({
  AgentChatWelcomePromptChips: () => null,
  AgentChatWelcomeRolePanel: () => null,
}))

describe("AgentChatConversationArea", () => {
  it("clips the outer conversation surface so focused quiz controls cannot scroll it", () => {
    const props = {
      chat: {
        isLoggedIn: true,
        messages: [
          { id: "user-a", role: "user", content: "Question" },
          { id: "assistant-a", role: "assistant", content: "Answer" },
        ],
      } as AgentChatContextValue,
      composerFocusVersion: 0,
      onAnswerProbe: vi.fn(),
      onExamplePromptSelect: vi.fn(),
      onNextActionPromptSubmit: vi.fn(),
      onSendMessage: vi.fn(),
      onUploadAttachments: vi.fn(),
      processPanel: {
        activeProcessMessage: null,
        closeProcessPanel: vi.fn(),
        openProcessPanel: vi.fn(),
      } as never,
      reportActions: {
        canUseConversation: false,
        exportReport: vi.fn(),
        exportTranscript: vi.fn(),
        isExportingTranscript: false,
        isGeneratingReport: false,
        openReportCenter: vi.fn(),
      } as never,
      requestedComposerPanel: {
        focusVersion: 0,
        panel: null,
        version: 0,
      } as never,
    }
    const { container, rerender } = render(
      <AgentChatConversationArea {...props} />
    )

    const conversationArea = container.firstElementChild as HTMLDivElement

    expect(conversationArea).toHaveAttribute(
      "data-agent-chat-conversation-area",
      "thread"
    )
    expect(conversationArea).toHaveClass("overflow-clip")
    expect(conversationArea).not.toHaveClass("overflow-hidden")

    conversationArea.scrollTop = 480
    fireEvent.scroll(conversationArea)

    expect(conversationArea.scrollTop).toBe(0)

    conversationArea.scrollTop = 480
    rerender(
      <AgentChatConversationArea {...props} composerFocusVersion={1} />
    )

    expect(conversationArea.scrollTop).toBe(0)
  })
})
