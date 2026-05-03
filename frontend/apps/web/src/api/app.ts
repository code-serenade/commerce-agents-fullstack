import { request } from "@workspace/services/api/base"
import type { UserInfo } from "@workspace/services/api/auth"
import { toRequestError } from "@workspace/services/errors/request-error"

export type AppUserStatus = "enabled" | "disabled"

export interface AppRoleResponse {
  id: number
  name: string
  code: string
  createdAt?: string
  updatedAt?: string
}

export interface AppUserResponse {
  userId: string
  displayId: string
  displayName: string
  remark?: string | null
  status: AppUserStatus
  createdAt?: string
  updatedAt?: string
  roles: AppRoleResponse[]
}

export interface RegisterAppUserRequest {
  displayId: string
  displayName: string
  remark?: string | null
}

export interface CurrentAppUserPermissionsResponse {
  userId: string
  roleCodes: string[]
  permissionCodes: string[]
}

export const registerAppUserApi = async (
  body: RegisterAppUserRequest
): Promise<AppUserResponse> => {
  return request<RegisterAppUserRequest, AppUserResponse>({
    method: "POST",
    url: "/api/app/register",
    body,
  })
}

export const getCurrentAppUserPermissionsApi =
  async (options?: {
    suppressGlobalError?: boolean
  }): Promise<CurrentAppUserPermissionsResponse> => {
    return request<undefined, CurrentAppUserPermissionsResponse>({
      method: "GET",
      url: "/api/app/me/permissions",
      suppressGlobalError: options?.suppressGlobalError,
    })
  }

export async function ensureCurrentAppUserPermissions(
  user: UserInfo
): Promise<CurrentAppUserPermissionsResponse> {
  try {
    return await getCurrentAppUserPermissionsApi({ suppressGlobalError: true })
  } catch (error) {
    if (!isMissingCurrentAppUserError(error)) throw error
  }

  await registerAppUserApi({
    displayId: user.display_id || user.id,
    displayName: user.name || user.display_id || user.id,
    remark: null,
  })

  return getCurrentAppUserPermissionsApi()
}

function isMissingCurrentAppUserError(error: unknown) {
  const requestError = toRequestError(error)

  return (
    requestError.code === -12000 ||
    requestError.status === 404 ||
    requestError.code === -10002
  )
}
