import type { AdminUserTableQuery } from "./types"

export const adminUsersQueryKey = ["admin", "admin-users", "rows"] as const

export const adminUserInitialQuery: AdminUserTableQuery = {
  keyword: "",
  status: "",
}

export const adminUserPageSizeOptions = [10, 20, 50] as const
