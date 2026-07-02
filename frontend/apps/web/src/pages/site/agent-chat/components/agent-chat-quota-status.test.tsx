import { render, screen } from "@testing-library/react"
import { MemoryRouter } from "react-router"
import { describe, expect, it } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import type { CurrentTeamUsageQuotaResponse } from "@/api"
import { AgentChatQuotaStatus } from "./agent-chat-quota-status"

describe("AgentChatQuotaStatus", () => {
  it("shows the 100 to 150 quota rule for unbound accounts", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatQuotaStatusShell i18n={i18n}>
        <AgentChatQuotaStatus
          isLoading={false}
          quota={createQuota({
            isSchoolTeamBound: false,
            questionLimit: 100,
            questionRemaining: 100,
          })}
        />
      </AgentChatQuotaStatusShell>
    )

    expect(screen.getByText("100 / 100 remaining")).toBeInTheDocument()
    expect(
      screen.getByText("100 without binding; 150 after binding")
    ).toBeInTheDocument()
    expect(screen.getByRole("link", { name: "Add info" })).toHaveAttribute(
      "href",
      "/profile"
    )
  })

  it("shows the 150 total quota for bound accounts", async () => {
    const i18n = await createAgentChatTestI18n()

    render(
      <AgentChatQuotaStatusShell i18n={i18n}>
        <AgentChatQuotaStatus
          isLoading={false}
          quota={createQuota({
            isSchoolTeamBound: true,
            questionLimit: 150,
            questionRemaining: 98,
          })}
        />
      </AgentChatQuotaStatusShell>
    )

    expect(screen.getByText("98 / 150 remaining")).toBeInTheDocument()
    expect(
      screen.getByText("School and team bound, 150 total")
    ).toBeInTheDocument()
    expect(screen.queryByRole("link", { name: "Add info" })).not.toBeInTheDocument()
  })
})

function AgentChatQuotaStatusShell({
  children,
  i18n,
}: {
  children: React.ReactNode
  i18n: Awaited<ReturnType<typeof createAgentChatTestI18n>>
}) {
  return (
    <MemoryRouter>
      <AgentChatTestI18nProvider i18n={i18n}>
        {children}
      </AgentChatTestI18nProvider>
    </MemoryRouter>
  )
}

function createQuota(
  overrides: Partial<CurrentTeamUsageQuotaResponse>
): CurrentTeamUsageQuotaResponse {
  return {
    userId: "user-1",
    displayId: "student-1",
    displayName: "Student 1",
    schoolName: "Unspecified school",
    teamName: "Unspecified team",
    isSchoolTeamBound: false,
    baseQuestionLimit: 100,
    schoolTeamBonusLimit: 50,
    questionLimit: 100,
    questionUsed: 0,
    questionRemaining: 100,
    status: "enabled",
    policyText: "",
    ...overrides,
  }
}
