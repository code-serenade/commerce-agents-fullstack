import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { normalizeNumericInput, SimpleFormDialog } from "./commerce-form"

describe("normalizeNumericInput", () => {
  it("keeps a non-negative decimal value to the configured precision", () => {
    const options = { decimalScale: 2, nonNegative: true }

    expect(normalizeNumericInput("12.345", options)).toBe("12.34")
    expect(normalizeNumericInput("-8.9", options)).toBe("8.9")
    expect(normalizeNumericInput("CNY 6.789", options)).toBe("6.78")
  })
})

describe("SimpleFormDialog", () => {
  it("keeps an optional review notice visible inside the form", async () => {
    const i18n = await createAgentChatTestI18n()
    const notice = i18n.t("commerce.common.smartFillReviewNotice")

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <SimpleFormDialog
          open
          title={i18n.t("commerce.products.dialogs.create")}
          fields={[
            {
              key: "name",
              label: i18n.t("commerce.fields.productName"),
              required: true,
            },
          ]}
          notice={notice}
          onOpenChange={vi.fn()}
          onSubmit={vi.fn()}
          setValues={vi.fn()}
          values={{ name: "" }}
        />
      </AgentChatTestI18nProvider>
    )

    expect(screen.getByText(notice)).toBeInTheDocument()
  })

  it("omits the notice block when no notice is provided", async () => {
    const i18n = await createAgentChatTestI18n()
    const notice = i18n.t("commerce.common.smartFillReviewNotice")

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <SimpleFormDialog
          open
          title={i18n.t("commerce.products.dialogs.create")}
          fields={[
            {
              key: "name",
              label: i18n.t("commerce.fields.productName"),
            },
          ]}
          onOpenChange={vi.fn()}
          onSubmit={vi.fn()}
          setValues={vi.fn()}
          values={{ name: "" }}
        />
      </AgentChatTestI18nProvider>
    )

    expect(screen.queryByText(notice)).not.toBeInTheDocument()
  })

  it("shows the draft process notice separately from the review notice", async () => {
    const i18n = await createAgentChatTestI18n()
    const notice = i18n.t("commerce.common.smartFillReviewNotice")
    const processNotice = i18n.t("commerce.common.smartFillProcess.agentRoute")

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <SimpleFormDialog
          open
          title={i18n.t("commerce.products.dialogs.create")}
          fields={[
            {
              key: "name",
              label: i18n.t("commerce.fields.productName"),
            },
          ]}
          notice={notice}
          processNotice={processNotice}
          onOpenChange={vi.fn()}
          onSubmit={vi.fn()}
          setValues={vi.fn()}
          values={{ name: "" }}
        />
      </AgentChatTestI18nProvider>
    )

    expect(screen.getByText(notice)).toBeInTheDocument()
    expect(screen.getByText(processNotice)).toBeInTheDocument()
  })
})
