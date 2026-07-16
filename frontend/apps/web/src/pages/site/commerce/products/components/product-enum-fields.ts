import type { SimpleField } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import { commerceEnumOptions } from "../../domain/enum-mappings"

export function assetFields(t: CommerceTranslate): SimpleField[] {
  return [
    {
      key: "assetType",
      label: t("commerce.fields.assetType"),
      options: commerceEnumOptions(t, "assetType", [
        "docs",
        "images",
        "videos",
        "other",
      ]),
      type: "select",
    },
    { key: "fileName", label: t("commerce.fields.fileName") },
    { key: "objectKey", label: t("commerce.fields.objectKey") },
    { key: "mimeType", label: t("commerce.fields.mimeType") },
    { key: "fileSize", label: t("commerce.fields.fileSize") },
    { key: "description", label: t("commerce.fields.note") },
  ]
}

export function materialFields(t: CommerceTranslate): SimpleField[] {
  return [
    {
      key: "materialType",
      label: t("commerce.fields.materialType"),
      options: commerceEnumOptions(t, "materialType", [
        "copy",
        "text",
        "image",
        "video",
        "document",
        "context",
        "excel",
      ]),
      required: true,
      type: "select",
    },
    { key: "title", label: t("commerce.fields.title") },
    { key: "content", label: t("commerce.fields.content"), type: "textarea" },
  ]
}

export function changeFields(t: CommerceTranslate): SimpleField[] {
  return [
    {
      key: "changeType",
      label: t("commerce.fields.changeType"),
      options: commerceEnumOptions(t, "changeType", [
        "price",
        "stock",
        "title",
        "description",
        "rating",
        "sales",
      ]),
      required: true,
      type: "select",
    },
    { key: "oldValue", label: t("commerce.fields.oldValue") },
    { key: "newValue", label: t("commerce.fields.newValue") },
    { key: "sourceType", label: t("commerce.fields.sourceType") },
    { key: "summary", label: t("commerce.fields.summary"), type: "textarea" },
  ]
}
