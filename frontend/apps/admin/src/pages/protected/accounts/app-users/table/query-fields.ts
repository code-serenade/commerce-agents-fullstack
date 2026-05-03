import type {
  DataTableBuiltInQueryField,
  DataTableQueryField,
} from "@workspace/app-kit"
import type { TFunction } from "i18next"
import type { AppUserTableQuery } from "../types"

export function buildAppUserStatusOptions(t: TFunction) {
  return [
    {
      label: t("admin.accounts.appUsers.table.status.enabled"),
      value: "enabled",
    },
    {
      label: t("admin.accounts.appUsers.table.status.disabled"),
      value: "disabled",
    },
  ] as const
}

export function buildAppUserBuiltInQueryFields(
  t: TFunction
): DataTableBuiltInQueryField<AppUserTableQuery>[] {
  return [
    {
      key: "keyword",
      type: "search",
      label: t("admin.accounts.appUsers.table.query.keyword.label"),
      placeholder: t("admin.accounts.appUsers.table.query.keyword.placeholder"),
    },
  ]
}

export function buildAppUserQueryFields(
  t: TFunction
): DataTableQueryField<AppUserTableQuery>[] {
  return [
    {
      key: "status",
      type: "select",
      label: t("admin.accounts.appUsers.table.query.status.label"),
      placeholder: t("admin.accounts.appUsers.table.query.status.placeholder"),
      options: buildAppUserStatusOptions(t).map((option) => ({
        label: option.label,
        value: option.value,
      })),
    },
  ]
}
