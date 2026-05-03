import { request } from "./base"
import { getAccessSignApi } from "./file"

export interface LoginRequest {
  identifier: string
  password: string
}

export interface LoginResponse {
  accessToken: string
  refreshToken: string
}

export interface RegisterRequest {
  username: string
  password: string
  display_name?: string
  email?: string
  avatar?: string
}

export interface UserInfo {
  id: string
  display_id: string
  email: string
  name: string
  avatar: string
}

export interface MeResponse {
  id: string
  display_user_id?: string | null
  username: string
  display_name?: string | null
  avatar?: string | null
  email?: string | null
  email_verified: boolean
  disabled: boolean
}

export interface UpdatePasswordPayload {
  oldPassword: string
  newPassword: string
}

export interface UpdateEmailPayload {
  email?: string | null
}

export interface UpdateProfilePayload {
  displayName?: string | null
  avatar?: string | null
}

export const loginApi = async (
  payload: LoginRequest
): Promise<LoginResponse> => {
  return request<LoginRequest, LoginResponse>({
    method: "POST",
    url: "/auth/session/login",
    body: payload,
    group: "auth",
  })
}

export const registerApi = async (payload: RegisterRequest): Promise<void> => {
  await request<RegisterRequest, unknown>({
    method: "POST",
    url: "/auth/session/register",
    body: payload,
    group: "auth",
  })
}

export const meApi = async (): Promise<UserInfo> => {
  const response = await request<undefined, MeResponse>({
    method: "GET",
    url: "/auth/user/me",
    group: "auth",
  })
  const avatar = await resolveAvatarUrl(response.avatar ?? "")

  return {
    id: response.id,
    display_id: response.display_user_id ?? "",
    email: response.email ?? "",
    name: response.display_name || response.username,
    avatar,
  }
}

async function resolveAvatarUrl(avatar: string): Promise<string> {
  if (!avatar || isResolvedAvatarUrl(avatar)) {
    return avatar
  }

  try {
    const sign = await getAccessSignApi({ key: avatar })
    return sign.download_url
  } catch {
    return avatar
  }
}

function isResolvedAvatarUrl(value: string): boolean {
  return (
    value.startsWith("http://") ||
    value.startsWith("https://") ||
    value.startsWith("blob:") ||
    value.startsWith("data:") ||
    value.startsWith("/")
  )
}

export const getAuthUserProfileApi = async (
  accessToken: string
): Promise<MeResponse> => {
  const response = await fetch("/auth/user/me", {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${accessToken}`,
    },
  })

  const json = (await response.json().catch(() => ({}))) as {
    data?: MeResponse
  }

  if (!response.ok || !json.data) {
    throw new Error("failed to fetch auth user profile")
  }

  return json.data
}

interface UpdateProfileRequestBody {
  display_name?: string | null
  avatar?: string | null
}

export const updatePasswordApi = async (
  payload: UpdatePasswordPayload
): Promise<void> => {
  await request<UpdatePasswordPayload, undefined>({
    method: "PUT",
    url: "/auth/user/password",
    body: payload,
    group: "auth",
  })
}

export const updateEmailApi = async (
  payload: UpdateEmailPayload
): Promise<void> => {
  await request<UpdateEmailPayload, undefined>({
    method: "PUT",
    url: "/auth/user/email",
    body: payload,
    group: "auth",
  })
}

export const updateProfileApi = async (
  payload: UpdateProfilePayload
): Promise<void> => {
  const body: UpdateProfileRequestBody = {
    display_name: payload.displayName,
    avatar: payload.avatar,
  }

  await request<UpdateProfileRequestBody, undefined>({
    method: "PUT",
    url: "/auth/user/profile",
    body,
    group: "auth",
  })
}

export const verifyEmailApi = async (): Promise<void> => {
  await request<undefined, undefined>({
    method: "POST",
    url: "/auth/user/email/verify",
    group: "auth",
  })
}
