import { useEffect, useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { useDispatch, useSelector } from "react-redux"
import { RouterProvider, createBrowserRouter, Navigate } from "react-router"
import { useQuery } from "@tanstack/react-query"
import {
  GlobalRequestError,
  PageLoading,
  addAuthProfileUserUpdatedListener,
} from "@workspace/app-kit"
import { loginSuccess, updateUser } from "@/store/authSlice"
import { authRoutes } from "@/routes/authRoutes"
import { siteRoutes } from "@/routes/siteRoutes"
import { ToastProvider } from "@workspace/ui-components/stable/toast"
import { ensureCurrentAppUserPermissions, meApi } from "./api"
import { toRequestError } from "@workspace/services/errors/request-error"
import { clearRequestErrorMessage } from "./store/requestErrorSlice"
import { resetAccess, setAccess } from "./store/accessSlice"
import type { RootState } from "./store"

const AUTH_RESTORE_TIMEOUT_MS = 12000
const APP_USER_DISABLED_CODE = -12001

export default function App() {
  const { t } = useTranslation()
  const requestErrorMessage = useSelector(
    (state: { requestError: { message: string | null } }) =>
      state.requestError.message
  )
  const isLogin = useSelector((state: RootState) => state.auth.isLogin)
  const dispatch = useDispatch()
  const token = localStorage.getItem("token")
  const [restoreTimedOutToken, setRestoreTimedOutToken] = useState<
    string | null
  >(null)
  const restoreTimedOut = Boolean(token && restoreTimedOutToken === token)

  const restoreQuery = useQuery({
    queryKey: ["auth", "restore", token],
    enabled: Boolean(token),
    retry: false,
    queryFn: async () => {
      const user = await meApi({ suppressGlobalError: true })
      const access = await ensureCurrentAppUserPermissions(user)

      return { user, access }
    },
  })

  useEffect(() => {
    if (!token || !restoreQuery.data) return
    dispatch(loginSuccess({ token, user: restoreQuery.data.user }))
    dispatch(
      setAccess({
        roleCodes: restoreQuery.data.access.roleCodes,
        permissionCodes: restoreQuery.data.access.permissionCodes,
      })
    )
  }, [dispatch, restoreQuery.data, token])

  useEffect(() => {
    if (!restoreQuery.isError) return
    const requestError = toRequestError(restoreQuery.error)
    console.warn(
      "🔁 Failed to restore session:",
      requestError
    )
    if (shouldClearStoredAuthOnRestoreError(requestError)) {
      localStorage.removeItem("token")
      localStorage.removeItem("refreshToken")
    }
    dispatch(resetAccess())
  }, [dispatch, restoreQuery.error, restoreQuery.isError])

  useEffect(() => {
    if (
      !token ||
      isLogin ||
      restoreQuery.isError ||
      restoreQuery.isSuccess ||
      restoreTimedOut
    ) {
      return
    }

    const timer = window.setTimeout(() => {
      console.warn("🔁 Session restore timed out, continuing without login")
      dispatch(resetAccess())
      setRestoreTimedOutToken(token)
    }, AUTH_RESTORE_TIMEOUT_MS)

    return () => window.clearTimeout(timer)
  }, [
    dispatch,
    isLogin,
    restoreQuery.isError,
    restoreQuery.isSuccess,
    restoreTimedOut,
    token,
  ])

  useEffect(() => {
    return addAuthProfileUserUpdatedListener((user) => {
      dispatch(updateUser(user))
    })
  }, [dispatch])

  const isRestored =
    !token ||
    isLogin ||
    restoreQuery.status === "error" ||
    restoreTimedOut
  const router = useMemo(() => {
    if (!isRestored) return null

    return createBrowserRouter([
      siteRoutes,
      authRoutes,
      {
        path: "*",
        element: <Navigate to="/" replace />,
      },
    ])
  }, [isRestored])

  if (!router) return <PageLoading fullscreen label={t("web.loading.app")} />

  return (
    <>
      <GlobalRequestError
        message={requestErrorMessage}
        closeLabel={t("actions.close", { ns: "common" })}
        onClose={() => dispatch(clearRequestErrorMessage())}
      />
      <RouterProvider router={router} />
      <ToastProvider position="top-center" />
    </>
  )
}

function shouldClearStoredAuthOnRestoreError(
  error: ReturnType<typeof toRequestError>
) {
  return error.status === 401 || error.code === APP_USER_DISABLED_CODE
}
