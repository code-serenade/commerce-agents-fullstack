import type {
  DataTableBuiltInQueryField,
  DataTableQueryField,
} from "@workspace/app-kit"
import type { TFunction } from "i18next"
import type { AdminUserTableQuery } from "../types"

export function buildAdminUserStatusOptions(t: TFunction) {
  return [
    {
      label: t("admin.accounts.adminUsers.table.status.enabled"),
      value: "enabled",
    },
    {
      label: t("admin.accounts.adminUsers.table.status.disabled"),
      value: "disabled",
    },
  ] as const
}

export function buildAdminUserBuiltInQueryFields(
  t: TFunction
): DataTableBuiltInQueryField<AdminUserTableQuery>[] {
  return [
    {
      key: "keyword",
      type: "search",
      label: t("admin.accounts.adminUsers.table.query.keyword.label"),
      placeholder: t(
        "admin.accounts.adminUsers.table.query.keyword.placeholder"
      ),
    },
  ]
}

export function buildAdminUserQueryFields(
  t: TFunction
): DataTableQueryField<AdminUserTableQuery>[] {
  return [
    {
      key: "status",
      type: "select",
      label: t("admin.accounts.adminUsers.table.query.status.label"),
      placeholder: t("admin.accounts.adminUsers.table.query.status.placeholder"),
      options: buildAdminUserStatusOptions(t).map((option) => ({
        label: option.label,
        value: option.value,
      })),
    },
  ]
}
