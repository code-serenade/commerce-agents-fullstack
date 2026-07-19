import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import type { AgentOutputBackendState } from "../use-agent-output-page"
import { OutputBackendDeliverables } from "./output-backend-deliverables"

describe("OutputBackendDeliverables", () => {
  it("opens a report detail on mobile and returns to the report list", async () => {
    const i18n = await createAgentChatTestI18n()
    const user = userEvent.setup()
    const setSelectedArtifactId = vi.fn()
    const workspace = {
      id: "workspace-1",
      status: "active",
      title: ["Mobile", "training", "task"].join(" "),
    }
    const artifact = {
      artifactType: "final_report",
      content: "# Mobile report\n\nReport content",
      format: "markdown",
      id: "artifact-1",
      source: "agent_session",
      status: "active",
      storageKind: "inline",
      title: ["Mobile", "report"].join(" "),
      validationStatus: "passed",
      workspace,
      workspaceId: workspace.id,
    }
    const backend: AgentOutputBackendState = {
      artifactExport: null,
      canExportTranscript: false,
      canGenerateReport: false,
      closeReportDialog: vi.fn(),
      context: {
        conversationId: null,
        runId: null,
        workspaceId: null,
      },
      currentConversationArtifactId: null,
      deliverables: [artifact],
      errorMessage: null,
      exportTranscript: vi.fn(),
      generateReport: vi.fn(),
      isExportLoading: false,
      isGeneratingReport: false,
      isReportDialogOpen: false,
      isTranscriptExporting: false,
      loading: false,
      openReportDialog: vi.fn(),
      refetch: vi.fn(),
      selectedArtifact: artifact,
      selectedArtifactId: artifact.id,
      setSelectedArtifactId,
      workspaces: [workspace],
    }

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <OutputBackendDeliverables backend={backend} />
      </AgentChatTestI18nProvider>
    )

    const list = screen.getByRole("complementary")
    const reportButton = screen.getByRole("button", {
      name: /Mobile report/,
    })
    const backButton = screen.getByRole("button", {
      name: i18n.t("agentOutput.backend.backToList"),
    })
    const detail = backButton.closest("section")

    expect(list).not.toHaveClass("hidden")
    expect(detail).toHaveClass("hidden")
    expect(backButton).toHaveAttribute("data-variant", "link")
    expect(backButton).toHaveClass("text-primary")
    expect(screen.getAllByText("Checked")[0].parentElement).toHaveClass(
      "[&_[data-slot=badge]]:bg-emerald-50"
    )

    const refreshButton = screen.getByRole("button", {
      name: i18n.t("agentOutput.backend.refresh"),
    })
    expect(refreshButton).toHaveAttribute("data-variant", "link")
    expect(refreshButton).toHaveClass("text-primary")

    await user.click(refreshButton)
    expect(backend.refetch).toHaveBeenCalledTimes(1)

    await user.click(reportButton)

    expect(setSelectedArtifactId).toHaveBeenCalledWith(artifact.id)
    expect(list).toHaveClass("hidden")
    expect(detail).not.toHaveClass("hidden")

    await user.click(backButton)

    expect(list).not.toHaveClass("hidden")
    expect(detail).toHaveClass("hidden")
  })
})
