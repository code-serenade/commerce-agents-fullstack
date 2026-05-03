import { useCallback } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import type {
  DataTableFetchResult,
  DataTableSortState,
} from "@workspace/app-kit"
import { listRolesApi } from "@/api"
import { rolesQueryKey } from "./constants"
import {
  filterRoles,
  paginateRoles,
  sortRoles,
} from "./table/logic"
import { compareRoleSortValues, getRoleSortValue } from "./table/sort"
import type { RoleRow, RoleTableQuery } from "./types"

export function useRolesData() {
  const queryClient = useQueryClient()

  const loadRoleRows = useCallback(async (): Promise<RoleRow[]> => {
    const roles = await listRolesApi()

    return roles.map((role) => ({
      id: role.id,
      name: role.name,
      code: role.code,
    }))
  }, [])

  useQuery({
    queryKey: rolesQueryKey,
    queryFn: loadRoleRows,
  })

  const fetchData = useCallback(
    async ({
      page,
      pageSize,
      query,
      signal,
      sort,
    }: {
      page: number
      pageSize: number
      query: RoleTableQuery
      signal: AbortSignal
      sort: DataTableSortState | null
    }): Promise<DataTableFetchResult<RoleRow>> => {
      void signal
      const rows = await queryClient.ensureQueryData({
        queryKey: rolesQueryKey,
        queryFn: loadRoleRows,
      })

      const filteredRows = filterRoles(rows, query)
      const sortedRows = sortRoles(
        filteredRows,
        sort,
        getRoleSortValue,
        compareRoleSortValues
      )

      return paginateRoles(sortedRows, page, pageSize)
    },
    [loadRoleRows, queryClient]
  )

  const invalidateRoles = useCallback(
    () =>
      queryClient.invalidateQueries({
        queryKey: rolesQueryKey,
      }),
    [queryClient]
  )

  return {
    fetchData,
    invalidateRoles,
  }
}
