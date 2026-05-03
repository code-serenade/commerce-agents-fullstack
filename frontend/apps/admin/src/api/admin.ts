import { request } from "@workspace/services/api/base"

export type AdminUserStatus = "enabled" | "disabled"
export type AppUserStatus = "enabled" | "disabled"
export type AppUserSortBy = "createdAt" | "updatedAt"
export type SortOrder = "asc" | "desc"

export interface AdminUserResponse {
  displayId?: string
  userId: string
  displayName?: string
  remark?: string | null
  status: AdminUserStatus
  adminRoles?: RoleResponse[]
  roles?: RoleResponse[]
}

export interface AppUserResponse {
  displayId?: string
  userId: string
  displayName?: string
  remark?: string | null
  status: AppUserStatus
  createdAt?: string | null
  updatedAt?: string | null
  roles: RoleResponse[]
}

export interface AppUserMetricsResponse {
  total: number
  enabled: number
  disabled: number
  multiRole: number
}

export interface PaginatedResponse<T> {
  items: T[]
  page: number
  pageSize: number
  total: number
  totalPages: number
  hasNext: boolean
  hasPrev: boolean
}

export interface CreateAdminUserRequest {
  identifier: string
  remark?: string | null
}

export interface UpdateAdminUserRequest {
  remark?: string | null
  status: AdminUserStatus
}

export interface UpdateAppUserRequest {
  remark?: string | null
  status: AppUserStatus
}

export interface RoleResponse {
  id: number
  name: string
  code: string
}

export interface CreateRoleRequest {
  name: string
  code: string
}

export type PermissionKind = "group" | "action"

export interface MenuResponse {
  id: number
  name: string
  parentId: number | null
  permissionCode: string | null
}

export interface CreateMenuRequest {
  name: string
  parentId?: number | null
  permissionCode?: string | null
}

export interface UserRoleOptionResponse {
  id: number
  name: string
  code: string
  checked: boolean
}

export type AppUserRoleOptionResponse = UserRoleOptionResponse

export interface UpdateRolePermissionsRequest {
  permissionIds: number[]
}

export interface UpdateUserRolesRequest {
  roleIds: number[]
}

export interface RolePermissionTreeNode {
  id: number
  code?: string | null
  name: string
  kind: PermissionKind
  checked: boolean
  children: RolePermissionTreeNode[]
}

export interface CurrentUserPermissionsResponse {
  userId: string
  roleCodes: string[]
  permissionCodes: string[]
}

export type CurrentAppUserPermissionsResponse = CurrentUserPermissionsResponse

export interface CurrentUserMenuNode {
  id: number
  name: string
  parentId: number | null
  permissionCode: string | null
  children: CurrentUserMenuNode[]
}

export const createAdminUserApi = async (
  body: CreateAdminUserRequest
): Promise<AdminUserResponse> => {
  return request<CreateAdminUserRequest, AdminUserResponse>({
    method: "POST",
    url: "/api/admin/account/admin-users",
    body,
  })
}

export const listAdminUsersApi = async (): Promise<AdminUserResponse[]> => {
  return request<undefined, AdminUserResponse[]>({
    method: "GET",
    url: "/api/admin/account/admin-users",
  })
}

export const updateAdminUserApi = async (
  userId: string,
  body: UpdateAdminUserRequest
): Promise<AdminUserResponse> => {
  return request<UpdateAdminUserRequest, AdminUserResponse>({
    method: "PATCH",
    url: `/api/admin/account/admin-users/${encodeURIComponent(userId)}`,
    body,
  })
}

export const deleteAdminUserApi = async (userId: string): Promise<void> => {
  return request<undefined, void>({
    method: "DELETE",
    url: `/api/admin/account/admin-users/${encodeURIComponent(userId)}`,
  })
}

export interface ListAppUsersRequest {
  page?: number
  pageSize?: number
  keyword?: string
  status?: AppUserStatus
  createdAtFrom?: string
  createdAtTo?: string
  updatedAtFrom?: string
  updatedAtTo?: string
  sortBy?: AppUserSortBy
  sortOrder?: SortOrder
}

export const listAppUsersApi = async (
  params: ListAppUsersRequest = {}
): Promise<PaginatedResponse<AppUserResponse>> => {
  return request<ListAppUsersRequest, PaginatedResponse<AppUserResponse>>({
    method: "GET",
    url: "/api/admin/account/app-users",
    body: params,
  })
}

export const listAppUserMetricsApi =
  async (): Promise<AppUserMetricsResponse> => {
    return request<undefined, AppUserMetricsResponse>({
      method: "GET",
      url: "/api/admin/account/app-users/metrics",
    })
  }

export const updateAppUserApi = async (
  userId: string,
  body: UpdateAppUserRequest
): Promise<AppUserResponse> => {
  return request<UpdateAppUserRequest, AppUserResponse>({
    method: "PATCH",
    url: `/api/admin/account/app-users/${encodeURIComponent(userId)}`,
    body,
  })
}

export const deleteAppUserApi = async (userId: string): Promise<void> => {
  return request<undefined, void>({
    method: "DELETE",
    url: `/api/admin/account/app-users/${encodeURIComponent(userId)}`,
  })
}

export const createRoleApi = async (
  body: CreateRoleRequest
): Promise<RoleResponse> => {
  return request<CreateRoleRequest, RoleResponse>({
    method: "POST",
    url: "/api/admin/access/roles",
    body,
  })
}

export const listRolesApi = async (): Promise<RoleResponse[]> => {
  return request<undefined, RoleResponse[]>({
    method: "GET",
    url: "/api/admin/access/roles",
  })
}

export const deleteRoleApi = async (roleId: number): Promise<void> => {
  return request<undefined, void>({
    method: "DELETE",
    url: `/api/admin/access/roles/${encodeURIComponent(String(roleId))}`,
  })
}

export const createAppRoleApi = async (
  body: CreateRoleRequest
): Promise<RoleResponse> => {
  return request<CreateRoleRequest, RoleResponse>({
    method: "POST",
    url: "/api/admin/access/app-roles",
    body,
  })
}

export const listAppRolesApi = async (): Promise<RoleResponse[]> => {
  return request<undefined, RoleResponse[]>({
    method: "GET",
    url: "/api/admin/access/app-roles",
  })
}

export const deleteAppRoleApi = async (roleId: number): Promise<void> => {
  return request<undefined, void>({
    method: "DELETE",
    url: `/api/admin/access/app-roles/${encodeURIComponent(String(roleId))}`,
  })
}

export const createMenuApi = async (
  body: CreateMenuRequest
): Promise<MenuResponse> => {
  return request<CreateMenuRequest, MenuResponse>({
    method: "POST",
    url: "/api/admin/access/menus",
    body,
  })
}

export const listMenusApi = async (): Promise<MenuResponse[]> => {
  return request<undefined, MenuResponse[]>({
    method: "GET",
    url: "/api/admin/access/menus",
  })
}

export const listUserRolesApi = async (
  userId: string
): Promise<UserRoleOptionResponse[]> => {
  return request<undefined, UserRoleOptionResponse[]>({
    method: "GET",
    url: `/api/admin/account/admin-users/${encodeURIComponent(userId)}/roles`,
  })
}

export const updateUserRolesApi = async (
  userId: string,
  body: UpdateUserRolesRequest
): Promise<UserRoleOptionResponse[]> => {
  return request<UpdateUserRolesRequest, UserRoleOptionResponse[]>({
    method: "PUT",
    url: `/api/admin/account/admin-users/${encodeURIComponent(userId)}/roles`,
    body,
  })
}

export const listAppUserRolesApi = async (
  userId: string
): Promise<AppUserRoleOptionResponse[]> => {
  return request<undefined, AppUserRoleOptionResponse[]>({
    method: "GET",
    url: `/api/admin/account/app-users/${encodeURIComponent(userId)}/roles`,
  })
}

export const updateAppUserRolesApi = async (
  userId: string,
  body: UpdateUserRolesRequest
): Promise<AppUserRoleOptionResponse[]> => {
  return request<UpdateUserRolesRequest, AppUserRoleOptionResponse[]>({
    method: "PUT",
    url: `/api/admin/account/app-users/${encodeURIComponent(userId)}/roles`,
    body,
  })
}

export const listRolePermissionsApi = async (
  roleId: number
): Promise<RolePermissionTreeNode[]> => {
  return request<undefined, RolePermissionTreeNode[]>({
    method: "GET",
    url: `/api/admin/access/roles/${encodeURIComponent(String(roleId))}/permissions`,
  })
}

export const updateRolePermissionsApi = async (
  roleId: number,
  body: UpdateRolePermissionsRequest
): Promise<RolePermissionTreeNode[]> => {
  return request<UpdateRolePermissionsRequest, RolePermissionTreeNode[]>({
    method: "PUT",
    url: `/api/admin/access/roles/${encodeURIComponent(String(roleId))}/permissions`,
    body,
  })
}

export const listAppRolePermissionsApi = async (
  roleId: number
): Promise<RolePermissionTreeNode[]> => {
  return request<undefined, RolePermissionTreeNode[]>({
    method: "GET",
    url: `/api/admin/access/app-roles/${encodeURIComponent(String(roleId))}/permissions`,
  })
}

export const updateAppRolePermissionsApi = async (
  roleId: number,
  body: UpdateRolePermissionsRequest
): Promise<RolePermissionTreeNode[]> => {
  return request<UpdateRolePermissionsRequest, RolePermissionTreeNode[]>({
    method: "PUT",
    url: `/api/admin/access/app-roles/${encodeURIComponent(String(roleId))}/permissions`,
    body,
  })
}

export const getCurrentUserPermissionsApi =
  async (): Promise<CurrentUserPermissionsResponse> => {
    return request<undefined, CurrentUserPermissionsResponse>({
      method: "GET",
      url: "/api/admin/me/permissions",
    })
  }

export const getCurrentUserMenusApi = async (): Promise<
  CurrentUserMenuNode[]
> => {
  return request<undefined, CurrentUserMenuNode[]>({
    method: "GET",
    url: "/api/admin/me/menus",
  })
}

export const getCurrentAppUserPermissionsApi =
  async (): Promise<CurrentAppUserPermissionsResponse> => {
    return request<undefined, CurrentAppUserPermissionsResponse>({
      method: "GET",
      url: "/api/app/me/permissions",
    })
  }
