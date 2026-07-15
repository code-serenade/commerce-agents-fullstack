import { useCallback } from "react"
import { useDispatch } from "react-redux"
import type { AuthLoginSubmitValues } from "@workspace/app-kit/login"
import { toRequestError } from "@workspace/services/errors/request-error"
import type { RequestError } from "@workspace/services/errors/request-error"
import { ensureCurrentAppUserPermissions, loginApi, meApi } from "@/api"
import { resetAccess, setAccess } from "@/store/accessSlice"
import { loginSuccess, logout } from "@/store/authSlice"
import type { LoginStage } from "@/pages/auth/login-error-copy"

export interface AuthLoginFlowError {
  requestError: RequestError
  stage: LoginStage
}

export function useAuthLoginFlow() {
  const dispatch = useDispatch()

  return useCallback(
    async (values: AuthLoginSubmitValues) => {
      let stage: LoginStage = "credentials"

      try {
        const result = await loginApi(values)
        localStorage.setItem("token", result.accessToken)
        localStorage.setItem("refreshToken", result.refreshToken)

        stage = "profile"
        const user = await meApi()
        stage = "workspace"
        const access = await ensureCurrentAppUserPermissions(user)

        dispatch(loginSuccess({ token: result.accessToken, user }))
        dispatch(
          setAccess({
            roleCodes: access.roleCodes,
            permissionCodes: access.permissionCodes,
          })
        )
      } catch (error) {
        const requestError = toRequestError(error)
        localStorage.removeItem("token")
        localStorage.removeItem("refreshToken")
        dispatch(logout())
        dispatch(resetAccess())
        throw { requestError, stage } satisfies AuthLoginFlowError
      }
    },
    [dispatch]
  )
}
