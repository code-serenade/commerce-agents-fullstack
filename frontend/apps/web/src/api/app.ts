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
  schoolName?: string | null
  teamName?: string | null
  remark?: string | null
  status: AppUserStatus
  createdAt?: string
  updatedAt?: string
  roles: AppRoleResponse[]
}

export interface RegisterAppUserRequest {
  displayId: string
  displayName: string
  schoolName?: string | null
  teamName?: string | null
  remark?: string | null
}

export interface CurrentAppUserPermissionsResponse {
  userId: string
  roleCodes: string[]
  permissionCodes: string[]
}

export type TeamUsageQuotaStatus = "enabled" | "disabled"

export interface CurrentTeamUsageQuotaResponse {
  userId: string
  displayId: string
  displayName: string
  schoolName: string
  teamName: string
  questionLimit: number
  questionUsed: number
  questionRemaining: number
  status: TeamUsageQuotaStatus
  policyText: string
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

export const getCurrentTeamUsageQuotaApi =
  async (options?: {
    suppressGlobalError?: boolean
  }): Promise<CurrentTeamUsageQuotaResponse> => {
    return request<undefined, CurrentTeamUsageQuotaResponse>({
      method: "GET",
      url: "/api/app/usage/quota",
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

  const pendingProfile = readPendingRegistrationProfile(user)

  await registerAppUserApi({
    displayId: user.display_id || user.id,
    displayName: user.name || user.display_id || user.id,
    schoolName: normalizeOptionalProfileValue(pendingProfile?.schoolName),
    teamName: normalizeOptionalProfileValue(pendingProfile?.teamName),
    remark: null,
  })
  clearPendingRegistrationProfile()

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

const PENDING_REGISTRATION_PROFILE_KEY =
  "commerce-agents.pendingRegistrationProfile"

export interface PendingRegistrationProfile {
  username: string
  displayName?: string
  email?: string
  schoolName?: string
  teamName?: string
  createdAt: string
}

export function savePendingRegistrationProfile(
  profile: Omit<PendingRegistrationProfile, "createdAt">
) {
  if (typeof window === "undefined") return
  const payload: PendingRegistrationProfile = {
    ...profile,
    createdAt: new Date().toISOString(),
  }
  window.localStorage.setItem(
    PENDING_REGISTRATION_PROFILE_KEY,
    JSON.stringify(payload)
  )
}

function readPendingRegistrationProfile(
  user: UserInfo
): PendingRegistrationProfile | null {
  if (typeof window === "undefined") return null
  const raw = window.localStorage.getItem(PENDING_REGISTRATION_PROFILE_KEY)
  if (!raw) return null

  try {
    const parsed = JSON.parse(raw) as PendingRegistrationProfile
    if (isPendingRegistrationProfileExpired(parsed.createdAt)) {
      clearPendingRegistrationProfile()
      return null
    }
    const userIdentifiers = new Set(
      [user.name, user.display_id, user.email].filter(isPresentString)
    )
    const pendingIdentifiers = [
      parsed.username,
      parsed.displayName,
      parsed.email,
    ].filter(isPresentString)
    if (
      userIdentifiers.size > 0 &&
      pendingIdentifiers.length > 0 &&
      !pendingIdentifiers.some((identifier) => userIdentifiers.has(identifier))
    ) {
      return null
    }
    return parsed
  } catch {
    clearPendingRegistrationProfile()
    return null
  }
}

function isPresentString(value: string | undefined | null): value is string {
  return Boolean(value)
}

function clearPendingRegistrationProfile() {
  if (typeof window === "undefined") return
  window.localStorage.removeItem(PENDING_REGISTRATION_PROFILE_KEY)
}

function normalizeOptionalProfileValue(value: string | undefined) {
  const normalized = value?.trim()
  return normalized ? normalized : null
}

function isPendingRegistrationProfileExpired(createdAt: string) {
  const createdAtMs = Date.parse(createdAt)
  if (!Number.isFinite(createdAtMs)) return true

  const maxAgeMs = 24 * 60 * 60 * 1000
  return Date.now() - createdAtMs > maxAgeMs
}
