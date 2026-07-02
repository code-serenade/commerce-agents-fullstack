import { useEffect, useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { useDispatch, useSelector } from "react-redux"
import {
  RouterProvider,
  createBrowserRouter,
  Navigate,
  useLocation,
} from "react-router"
import { useQuery } from "@tanstack/react-query"
import {
  GlobalRequestError,
  PageLoading,
  addAuthProfileUserUpdatedListener,
} from "@workspace/app-kit"
import { loginSuccess, updateUser } from "@/store/authSlice"
import { resetAccess, setAccess } from "@/store/accessSlice"
import type { RootState } from "@/store"
import { publicRoutes } from "@/routes/publicRoutes"
import { protectedRoutes } from "@/routes/protectedRoutes"
import { ToastProvider } from "@workspace/ui-components/stable/toast"
import { getCurrentUserPermissionsApi, meApi } from "./api"
import { toRequestError } from "@workspace/services/errors/request-error"
import { clearRequestErrorMessage } from "./store/requestErrorSlice"

const AUTH_RESTORE_TIMEOUT_MS = 12000
const ADMIN_USER_NOT_FOUND_CODE = -11000
const ADMIN_USER_DISABLED_CODE = -11001
const ADMIN_PERMISSION_DENIED_CODE = -11002
const ADMIN_AUTH_USER_NOT_FOUND_CODE = -11004

export default function App() {
  const { t } = useTranslation()
  const isLogin = useSelector((state: RootState) => state.auth.isLogin)
  const requestErrorMessage = useSelector(
    (state: RootState) => state.requestError.message
  )
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
      const [user, access] = await Promise.all([
        meApi(),
        getCurrentUserPermissionsApi(),
      ])

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
      console.warn("🔁 Admin session restore timed out, continuing to login")
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
    !token || restoreQuery.status !== "pending" || restoreTimedOut
  const isAuthenticated =
    isLogin || (Boolean(token) && restoreQuery.status === "success")

  const router = useMemo(() => {
    if (!isRestored) return null

    return createBrowserRouter([
      isAuthenticated ? protectedRoutes : publicRoutes,
      {
        path: "*",
        element: isAuthenticated ? (
          <Navigate to="/" replace />
        ) : (
          <RedirectToLoginWithTarget />
        ),
      },
    ])
  }, [isAuthenticated, isRestored])

  if (!router) return <PageLoading fullscreen label={t("admin.loading.app")} />

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

function RedirectToLoginWithTarget() {
  const location = useLocation()

  return (
    <Navigate
      to="/login"
      replace
      state={{
        from: {
          hash: location.hash,
          pathname: location.pathname,
          search: location.search,
        },
      }}
    />
  )
}

function shouldClearStoredAuthOnRestoreError(
  error: ReturnType<typeof toRequestError>
) {
  return (
    error.status === 401 ||
    error.code === ADMIN_USER_NOT_FOUND_CODE ||
    error.code === ADMIN_USER_DISABLED_CODE ||
    error.code === ADMIN_PERMISSION_DENIED_CODE ||
    error.code === ADMIN_AUTH_USER_NOT_FOUND_CODE
  )
}
