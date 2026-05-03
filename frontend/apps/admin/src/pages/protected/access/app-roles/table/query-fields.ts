import type { DataTableBuiltInQueryField } from "@workspace/app-kit"
import type { TFunction } from "i18next"
import type { RoleTableQuery } from "../types"

export function buildRoleBuiltInQueryFields(
  t: TFunction
): DataTableBuiltInQueryField<RoleTableQuery>[] {
  return [
    {
      key: "keyword",
      type: "search",
      label: t("admin.access.appRoles.table.query.keyword.label"),
      placeholder: t("admin.access.appRoles.table.query.keyword.placeholder"),
    },
  ]
}
