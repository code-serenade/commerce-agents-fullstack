import type { RoleRow, RoleTableQuery } from "../types"

export function filterRoles(
  rows: readonly RoleRow[],
  query: RoleTableQuery
) {
  return rows.filter((row) => {
    const keyword = query.keyword.trim().toLowerCase()
    const searchCandidates = [String(row.id), row.name, row.code]

    const matchesKeyword =
      keyword.length === 0 ||
      searchCandidates.some((candidate) =>
        candidate.toLowerCase().includes(keyword)
      )

    return matchesKeyword
  })
}

export function sortRoles(
  rows: readonly RoleRow[],
  sort: { columnKey: string; direction: "asc" | "desc" } | null,
  getSortValue: (row: RoleRow, columnKey: string) => string | number,
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

export function paginateRoles(
  rows: readonly RoleRow[],
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
