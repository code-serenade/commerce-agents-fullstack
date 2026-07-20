import { render, screen } from "@testing-library/react"
import { MemoryRouter } from "react-router"
import { describe, expect, it, vi } from "vitest"
import type { AgentRuntimeEvent } from "@/api"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import {
  node,
  reported,
} from "../view-model/agent-process-facts-test-helpers"
import { AgentProcessDrawer } from "./agent-process-drawer"

const mockAgentChatContext = vi.hoisted(() => ({
  feedback: {
    canSubmit: false,
    comment: "",
    disabledReason: null,
    error: null,
    isSubmitting: false,
    rating: 5,
    setComment: vi.fn(),
    setRating: vi.fn(),
    submitted: false,
    submit: vi.fn(async () => {}),
    target: null,
  },
  messages: [
    {
      content: "judge whether a summer sun-protection jacket can be tested",
      id: "user-1",
      role: "user",
    },
  ],
  task: "judge whether a summer sun-protection jacket can be tested",
}))

vi.mock("../context/use-agent-chat-context", () => ({
  useAgentChatContext: () => mockAgentChatContext,
}))

const task = mockAgentChatContext.task

describe("AgentProcessDrawer", () => {
  it(
    "shows pixel role workers inside the process dialog",
    async () => {
      const i18n = await createAgentChatTestI18n()

      render(
        <AgentChatTestI18nProvider i18n={i18n}>
          <MemoryRouter>
            <AgentProcessDrawer
              isStreaming={false}
              messageId="assistant-1"
              onClose={vi.fn()}
              runtimeEvents={createRuntimeEvents()}
              task={task}
            />
          </MemoryRouter>
        </AgentChatTestI18nProvider>
      )

      const dialog = screen.getByRole("dialog", {
        name: i18n.t("agentChat.workbench.title"),
      })
      const drawer = dialog.closest("[data-agent-process-drawer='true']")
      const roleCluster = dialog.querySelector("[data-agent-role-cluster]")

      expect(drawer).not.toBeNull()
      expect(drawer).toHaveClass("fixed")
      expect(drawer).toHaveClass("inset-0")
      expect(roleCluster).not.toBeNull()
      expect(
        dialog.querySelector("[data-agent-role-card='data']")
      ).not.toBeNull()
      expect(
        dialog.querySelector("[data-agent-portrait-asset='data-portrait']")
      ).not.toBeNull()
      expect(
        dialog.querySelector("[data-agent-portrait-asset='ops-portrait']")
      ).not.toBeNull()
      expect(
        screen.getByText(i18n.t("agentChat.thread.dispatchPanel.roleClusterTitle"))
      ).toBeInTheDocument()
    },
    15000
  )
})

function createRuntimeEvents(): AgentRuntimeEvent[] {
  return [
    {
      difficulty: "complex",
      planned_steps: [],
      primary_runtime_role: "data",
      route_mode: "task_graph",
      selected_skills: [],
      student_visible_explanation: "",
      support_runtime_roles: ["ops"],
      type: "dispatch_planned",
    },
    {
      graph: {
        graph_id: "graph-1",
        nodes: [
          node("data", "data"),
          node("ops", "ops"),
        ],
        root_task: task,
      },
      type: "task_graph_planned",
    },
    reported("data", "data"),
    reported("ops", "ops"),
    {
      content: "final answer",
      type: "output",
    },
  ]
}
