import { describe, expect, it } from "vitest"
import i18next from "i18next"
import zhMessages from "@/../lang/zhCN/messages"
import {
  commerceBooleanOptions,
  fileKindFilterOptions,
  getCommerceEnumLabel,
  knowledgeContentTypeLabel,
} from "./enum-mappings"

describe("getCommerceEnumLabel", () => {
  it("translates known business enum values", async () => {
    const i18n = await createTestI18n()
    expect(getCommerceEnumLabel(i18n.t, "campaignType", "promotion")).toBe(
      zhMessages.commerce.enumLabels.campaignType.promotion
    )
    expect(getCommerceEnumLabel(i18n.t, "severity", "warning")).toBe(
      zhMessages.commerce.enumLabels.severity.warning
    )
  })

  it("does not expose unknown enum codes", async () => {
    const i18n = await createTestI18n()
    expect(getCommerceEnumLabel(i18n.t, "adapterStatus", "future_code")).toBe(
      zhMessages.commerce.enumLabels.other
    )
  })

  it("exposes specialized mappings through the same public entry", async () => {
    const i18n = await createTestI18n()

    expect(commerceBooleanOptions(i18n.t)).toEqual([
      { label: zhMessages.commerce.common.no, value: "false" },
      { label: zhMessages.commerce.common.yes, value: "true" },
    ])
    expect(
      fileKindFilterOptions(i18n.t).find(({ value }) => value === "image")
        ?.label
    ).toBe(zhMessages.commerce.files.filters.kind.image)
    expect(knowledgeContentTypeLabel(i18n.t, "selling_point")).toBe(
      zhMessages.commerce.products.knowledgeContentTypes.sellingPoint
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
