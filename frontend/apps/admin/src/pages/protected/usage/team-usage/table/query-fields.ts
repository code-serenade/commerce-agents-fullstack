import type {
  DataTableBuiltInQueryField,
  DataTableQueryField,
} from "@workspace/app-kit"
import type { TFunction } from "i18next"
import type { TeamUsageTableQuery } from "../types"

export function buildTeamUsageStatusOptions(t: TFunction) {
  return [
    {
      label: t("admin.usage.teamUsage.table.status.enabled"),
      value: "enabled",
    },
    {
      label: t("admin.usage.teamUsage.table.status.disabled"),
      value: "disabled",
    },
  ] as const
}

export function buildTeamUsageBuiltInQueryFields(
  t: TFunction
): DataTableBuiltInQueryField<TeamUsageTableQuery>[] {
  return [
    {
      key: "keyword",
      type: "search",
      label: t("admin.usage.teamUsage.table.query.keyword.label"),
      placeholder: t(
        "admin.usage.teamUsage.table.query.keyword.placeholder"
      ),
    },
  ]
}

export function buildTeamUsageQueryFields(
  t: TFunction
): DataTableQueryField<TeamUsageTableQuery>[] {
  return [
    {
      key: "status",
      type: "select",
      label: t("admin.usage.teamUsage.table.query.status.label"),
      placeholder: t("admin.usage.teamUsage.table.query.status.placeholder"),
      options: buildTeamUsageStatusOptions(t).map((option) => ({
        label: option.label,
        value: option.value,
      })),
    },
  ]
}
