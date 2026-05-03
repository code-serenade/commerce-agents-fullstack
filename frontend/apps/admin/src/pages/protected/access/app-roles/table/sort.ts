import type { RoleRow } from "../types"

export function getRoleSortValue(row: RoleRow, columnKey: string) {
  switch (columnKey) {
    case "id":
      return row.id
    case "name":
      return row.name
    case "code":
      return row.code
    default:
      return ""
  }
}

export function compareRoleSortValues(
  left: string | number,
  right: string | number
) {
  if (typeof left === "number" && typeof right === "number") {
    return left - right
  }

  return String(left).localeCompare(String(right), "zh-CN")
}
