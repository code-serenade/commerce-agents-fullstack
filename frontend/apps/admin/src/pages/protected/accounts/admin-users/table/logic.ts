import type {
  AdminUserRow,
  AdminUserStatusFilter,
  AdminUserTableQuery,
} from "../types"
import { mapStatusLabelToApiStatus } from "./status"

export function filterAdminUsers(
  rows: readonly AdminUserRow[],
  query: AdminUserTableQuery
) {
  return rows.filter((row) => {
    const keyword = query.keyword.trim().toLowerCase()
    const searchCandidates = [
      row.display_name,
      row.display_id,
      row.user_id,
      row.remark ?? "",
      ...row.roles,
    ]

    const matchesKeyword =
      keyword.length === 0 ||
      searchCandidates.some((candidate) =>
        candidate.toLowerCase().includes(keyword)
      )

    const matchesStatus =
      query.status.length === 0 ||
      mapStatusLabelToApiStatus(query.status as AdminUserStatusFilter) === row.status

    return matchesKeyword && matchesStatus
  })
}

export function sortAdminUsers(
  rows: readonly AdminUserRow[],
  sort: { columnKey: string; direction: "asc" | "desc" } | null,
  getSortValue: (row: AdminUserRow, columnKey: string) => string | number,
  compareSortValues: (left: string | number, right: string | number) => number
) {
  if (!sort) return [...rows]

  return [...rows].sort((left, right) => {
    const leftValue = getSortValue(left, sort.columnKey)
    const rightValue = getSortValue(right, sort.columnKey)
    const result = compareSortValues(leftValue, rightValue)

    return sort.direction === "asc" ? result : -result
  })
}

export function paginateAdminUsers(
  rows: readonly AdminUserRow[],
  page: number,
  pageSize: number
) {
  const start = (page - 1) * pageSize
  const end = start + pageSize

  return {
    items: rows.slice(start, end),
    total: rows.length,
  }
}
