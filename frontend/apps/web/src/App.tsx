import { useEffect, useMemo } from "react"
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

export default function App() {
  const { t } = useTranslation()
  const requestErrorMessage = useSelector(
    (state: { requestError: { message: string | null } }) =>
      state.requestError.message
  )
  const dispatch = useDispatch()
  const token = localStorage.getItem("token")

  const restoreQuery = useQuery({
    queryKey: ["auth", "restore", token],
    enabled: Boolean(token),
    retry: false,
    queryFn: async () => {
      const user = await meApi()
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
    console.warn(
      "🔁 Failed to restore session:",
      toRequestError(restoreQuery.error)
    )
    localStorage.removeItem("token")
    localStorage.removeItem("refreshToken")
    dispatch(resetAccess())
  }, [dispatch, restoreQuery.error, restoreQuery.isError])

  useEffect(() => {
    return addAuthProfileUserUpdatedListener((user) => {
      dispatch(updateUser(user))
    })
  }, [dispatch])

  const isRestored = !token || restoreQuery.status !== "pending"

  const router = useMemo(() => {
    if (!isRestored) return null

    return createBrowserRouter([
      authRoutes,
      siteRoutes,
      {
        path: "*",
        element: <Navigate to="/home" replace />,
      },
    ])
  }, [isRestored])

  if (!router) return <PageLoading fullscreen />

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
