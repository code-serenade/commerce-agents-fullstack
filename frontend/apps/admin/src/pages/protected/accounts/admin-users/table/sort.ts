import type { AdminUserRow } from "../types"

export function getAdminUserSortValue(row: AdminUserRow, columnKey: string) {
  switch (columnKey) {
    case "display_name":
      return row.display_name
    case "display_id":
      return row.display_id
    case "remark":
      return row.remark ?? ""
    case "status":
      return row.status
    case "roleCount":
      return row.roles.length
    default:
      return ""
  }
}

export function compareSortValues(left: string | number, right: string | number) {
  if (typeof left === "number" && typeof right === "number") {
    return left - right
  }

  return String(left).localeCompare(String(right), "zh-CN")
}
