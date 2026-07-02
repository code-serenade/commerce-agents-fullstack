import { renderHook } from "@testing-library/react"
import { createElement, type ReactNode } from "react"
import { describe, expect, it } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { useRegisterPage } from "./use-register-page"

describe("useRegisterPage", () => {
  it("keeps school and team optional while explaining the 100 to 150 quota rule", async () => {
    const i18n = await createAgentChatTestI18n()

    const { result } = renderHook(() => useRegisterPage(), {
      wrapper: ({ children }: { children: ReactNode }) =>
        createElement(AgentChatTestI18nProvider, { i18n, children }),
    })

    const extraFields = result.current.extraFields ?? []
    const schoolField = extraFields.find(
      (field) => field.name === "schoolName"
    )
    const teamField = extraFields.find(
      (field) => field.name === "teamName"
    )

    expect(schoolField).toMatchObject({
      required: false,
      maxLength: 128,
    })
    expect(schoolField?.helpText).toContain("100")
    expect(schoolField?.helpText).toContain("150")
    expect(teamField).toMatchObject({
      required: false,
      maxLength: 128,
    })
    expect(teamField?.helpText).toContain("100")
  })
})
