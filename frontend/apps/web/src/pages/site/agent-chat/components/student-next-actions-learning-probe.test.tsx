import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { StudentNextActions } from "./thread-message-blocks"

vi.mock("./student-learning-probe-slot", () => ({
  StudentLearningProbeSlot: ({ messageId }: { messageId: string }) => (
    <div data-testid="learning-probe-slot">{messageId}</div>
  ),
}))

describe("StudentNextActions learning probe", () => {
  it("mounts only after the assistant message is persisted", async () => {
    const i18n = await createAgentChatTestI18n()
    const message = {
      id: "assistant-1787804209405-pdwt1iy9:0",
      role: "assistant" as const,
      content: "Completed commerce guidance",
    }
    const props = {
      conversationId: "conversation-a",
      message,
      onAnswerProbe: vi.fn(),
      showLearningProbe: true,
    }

    const { rerender } = render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <StudentNextActions {...props} />
      </AgentChatTestI18nProvider>
    )

    expect(screen.queryByTestId("learning-probe-slot")).not.toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.nextActions.title"))
    ).toBeInTheDocument()

    rerender(
      <AgentChatTestI18nProvider i18n={i18n}>
        <StudentNextActions
          {...props}
          message={{
            ...message,
            persistedId: "4dd12397-39f2-453c-a5fc-0816f50e0098",
          }}
        />
      </AgentChatTestI18nProvider>
    )

    expect(screen.getByTestId("learning-probe-slot")).toHaveTextContent(
      "4dd12397-39f2-453c-a5fc-0816f50e0098"
    )
    expect(
      screen
        .getByTestId("learning-probe-slot")
        .compareDocumentPosition(
          screen.getByText(i18n.t("agentChat.thread.nextActions.title"))
        ) & Node.DOCUMENT_POSITION_FOLLOWING
    ).toBeTruthy()
  })
})
