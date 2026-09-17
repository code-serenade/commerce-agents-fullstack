import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { MemoryRouter, Route, Routes, useNavigate } from "react-router"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { StudentLearningProbe } from "./student-learning-probe"

const QUESTION = "Which traffic metric would you check first?"

function LearningProgressTestRoute() {
  const navigate = useNavigate()
  return (
    <div>
      <p>Learning progress route</p>
      <button type="button" onClick={() => navigate(-1)}>
        Back to chat
      </button>
    </div>
  )
}

describe("StudentLearningProbe result navigation", () => {
  it("opens the learning record and restores the feedback when navigating back", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()
    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <MemoryRouter initialEntries={["/commerce/conversation-a"]}>
          <Routes>
            <Route
              path="/commerce/:conversationId"
              element={
                <StudentLearningProbe
                  onAnswer={vi.fn()}
                  onEngage={vi.fn().mockResolvedValue(true)}
                  onSkip={vi.fn()}
                  probe={{
                    engaged: true,
                    id: "probe-record-route",
                    questionText: QUESTION,
                  }}
                  ready
                  result={{
                    demonstratedPoints: [],
                    id: "probe-record-route",
                    outcome: "correct",
                    sourceLabels: [],
                    status: "answered",
                  }}
                />
              }
            />
            <Route
              path="/learning-progress"
              element={<LearningProgressTestRoute />}
            />
          </Routes>
        </MemoryRouter>
      </AgentChatTestI18nProvider>
    )

    await user.click(
      screen.getByRole("button", {
        name: i18n.t("agentChat.thread.learningProbe.viewRecord"),
      })
    )
    expect(screen.getByText("Learning progress route")).toBeVisible()

    await user.click(screen.getByRole("button", { name: "Back to chat" }))
    expect(screen.getByText(QUESTION)).toBeVisible()
    expect(
      screen.getByText(
        i18n.t("agentChat.thread.learningProbe.outcomeStatus.correct")
      )
    ).toBeVisible()
  })
})
