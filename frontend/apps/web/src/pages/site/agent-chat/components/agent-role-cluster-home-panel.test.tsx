import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { AgentRoleClusterHomePanel } from "./agent-role-cluster-home-panel"

const invalidAdRoleName = [
  0x5343, 0x5ddd, 0x6295, 0x653e, 0x4e13, 0x5458,
]
  .map((codePoint) => String.fromCodePoint(codePoint))
  .join("")

describe("AgentRoleClusterHomePanel", () => {
  it("shows the first-screen commerce agent preview", async () => {
    const i18n = await createAgentChatTestI18n()

    const { container } = render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <AgentRoleClusterHomePanel />
      </AgentChatTestI18nProvider>
    )

    expect(
      screen.getByText(i18n.t("agentChat.homeRoleCluster.title"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.homeRoleCluster.summary"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.homeRoleCluster.footer"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.homeRoleCluster.eyebrow"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.agents.data.name"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.agents.search.name"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.agents.creative.name"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.agents.design.name"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.agents.ops.summary"))
    ).toBeInTheDocument()
    expect(
      screen.getByText(i18n.t("agentChat.thread.agents.design.summary"))
    ).toBeInTheDocument()
    expect(
      screen.queryByText(i18n.t("agentChat.thread.roleClusterActivity.working.ops"))
    ).not.toBeInTheDocument()
    expect(screen.queryByText(invalidAdRoleName)).not.toBeInTheDocument()
    expect(
      container.querySelector("[data-agent-portrait-asset='ops-portrait']")
    ).not.toBeNull()
    expect(
      container.querySelector("[data-agent-portrait-asset='ops-portrait']")
    ).toHaveClass("object-contain")
    expect(
      container
        .querySelector("[data-agent-portrait-asset='ops-portrait']")
        ?.closest("[data-agent-portrait-frame='rect']")
    ).not.toBeNull()
    expect(
      container.querySelector("[data-agent-portrait-asset='ops-portrait']")
    ).not.toHaveClass("agent-role-portrait__sprite--working")
    expect(
      container.querySelector("[data-agent-portrait-asset='ops-portrait']")
    ).toHaveAttribute("data-agent-portrait-source", "static")
    expect(
      container.querySelector("[data-agent-portrait-asset='data-portrait']")
    ).not.toBeNull()
    expect(
      container.querySelector("[data-agent-portrait-asset='creative-portrait']")
    ).not.toBeNull()
    expect(
      container.querySelector("[data-agent-portrait-asset='creative-portrait']")
    ).toHaveAttribute("data-agent-portrait-source", "static")
    expect(
      container.querySelector("[data-agent-portrait-asset='design-portrait']")
    ).toHaveAttribute("data-agent-portrait-source", "static")
    expect(
      container.querySelector("[data-agent-portrait-asset='accounting-portrait']")
    ).toBeNull()
    expect(
      container.querySelector("[data-agent-portrait-asset='engineering-portrait']")
    ).toBeNull()
    expect(
      container.querySelector("[data-agent-portrait-asset='service-portrait']")
    ).toBeNull()
    expect(
      container.querySelector("[data-agent-portrait-asset='design-portrait']")
    ).not.toBeNull()
    expect(
      container.querySelector("[data-agent-portrait-asset='web-portrait']")
    ).toBeNull()
    expect(
      container.querySelectorAll("[data-agent-role-card]").length
    ).toBe(5)
    ;[
      "ops",
      "search",
      "data",
      "creative",
      "design",
    ].forEach((role) => {
      expect(
        container.querySelector(`[data-agent-role-card='${role}']`)
      ).not.toBeNull()
    })
    expect(
      container.querySelector("[data-agent-role-card='douyin_ads']")
    ).toBeNull()
    ;["service", "engineering", "accounting", "web"].forEach((role) => {
      expect(
        container.querySelector(`[data-agent-role-card='${role}']`)
      ).toBeNull()
    })
  })
})
