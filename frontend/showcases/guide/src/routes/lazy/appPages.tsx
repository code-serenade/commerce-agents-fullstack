import { lazy } from "react"

export const ButtonGuidePage = lazy(() => import("@/pages/ButtonGuidePage"))
export const DisplayGuidePage = lazy(() => import("@/pages/DisplayGuidePage"))
export const FeedbackGuidePage = lazy(() => import("@/pages/FeedbackGuidePage"))
export const FormGuidePage = lazy(() => import("@/pages/FormGuidePage"))
export const NavigationGuidePage = lazy(
  () => import("@/pages/NavigationGuidePage")
)

export const DataTableGuidePage = lazy(
  () => import("@/pages/DataTableGuidePage")
)

export const ThemeGuidePage = lazy(() => import("@/pages/ThemeGuidePage"))
