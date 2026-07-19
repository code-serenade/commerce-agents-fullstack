import { describe, expect, it } from "vitest"
import i18next from "i18next"
import zhMessages from "@/../lang/zhCN/messages"
import { getCommerceStatusLabel } from "./enum-mappings"

describe("getCommerceStatusLabel", () => {
  it("translates known statuses", async () => {
    const i18n = await createTestI18n()
    expect(getCommerceStatusLabel(i18n.t, "archived")).toBe(
      zhMessages.commerce.statusOptions.archived
    )
    expect(getCommerceStatusLabel(i18n.t, "paused")).toBe(
      zhMessages.commerce.statusOptions.paused
    )
  })

  it("does not expose unknown status codes", async () => {
    const i18n = await createTestI18n()
    expect(getCommerceStatusLabel(i18n.t, "future_status")).toBe(
      zhMessages.commerce.enumLabels.other
    )
  })
})

async function createTestI18n() {
  const i18n = i18next.createInstance()
  await i18n.init({
    lng: "zh-CN",
    resources: { "zh-CN": { translation: zhMessages } },
  })
  return i18n
}
