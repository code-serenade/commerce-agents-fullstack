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
import { resetAccess, setAccess } from "@/store/accessSlice"
import type { RootState } from "@/store"
import { publicRoutes } from "@/routes/publicRoutes"
import { protectedRoutes } from "@/routes/protectedRoutes"
import { ToastProvider } from "@workspace/ui-components/stable/toast"
import { getCurrentUserPermissionsApi, meApi } from "./api"
import { toRequestError } from "@workspace/services/errors/request-error"
import { clearRequestErrorMessage } from "./store/requestErrorSlice"

export default function App() {
  const { t } = useTranslation()
  const isLogin = useSelector((state: RootState) => state.auth.isLogin)
  const requestErrorMessage = useSelector(
    (state: RootState) => state.requestError.message
  )
  const dispatch = useDispatch()
  const token = localStorage.getItem("token")

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
  const isAuthenticated =
    isLogin || (Boolean(token) && restoreQuery.status === "success")

  const router = useMemo(() => {
    if (!isRestored) return null

    return createBrowserRouter([
      isAuthenticated ? protectedRoutes : publicRoutes,
      {
        path: "*",
        element: <Navigate to={isAuthenticated ? "/" : "/login"} replace />,
      },
    ])
  }, [isAuthenticated, isRestored])

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
