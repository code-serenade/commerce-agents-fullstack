import type { TFunction } from "i18next"

const routeKeyByPath: Record<string, string> = {
  "/": "commerce",
  "/agent": "commerce",
  "/commerce": "commerce",
  "/login": "login",
  "/register": "register",
  "/home": "home",
  "/overview": "overview",
  "/materials": "materials",
  "/products": "products",
  "/campaigns": "campaigns",
  "/workspaces": "workspaces",
  "/platform-connect": "platformConnect",
  "/approvals": "approvals",
  "/operations": "operations",
  "/agent-output": "agentOutput",
  "/agent-capabilities": "agentCapabilities",
  "/question-ideas": "questionIdeas",
  "/profile": "profile",
}

export interface RouteDisplayInfo {
  description: string
  hasExtraParams: boolean
  label: string
  normalizedPath: string
  rawPath: string
}

export function resolveRouteDisplayInfo(
  routePath: string,
  t: TFunction
): RouteDisplayInfo {
  const normalizedPath = normalizeRoutePath(routePath)
  const routeKey = routeKeyByPath[normalizedPath]
  const labelKey = routeKey
    ? `admin.analytics.routes.known.${routeKey}.label`
    : "admin.analytics.routes.unknown.label"
  const descriptionKey = routeKey
    ? `admin.analytics.routes.known.${routeKey}.description`
    : "admin.analytics.routes.unknown.description"
  const label = t(labelKey)
  const description = t(descriptionKey)

  return {
    description:
      description === descriptionKey ? t("admin.analytics.routes.unknown.description") : description,
    hasExtraParams: routePath !== normalizedPath,
    label: label === labelKey ? normalizedPath : label,
    normalizedPath,
    rawPath: routePath,
  }
}

function normalizeRoutePath(routePath: string) {
  const trimmed = routePath.trim()
  if (!trimmed) return "/"

  const queryIndex = trimmed.indexOf("?")
  const hashIndex = trimmed.indexOf("#")
  const cutIndexes = [queryIndex, hashIndex].filter((index) => index >= 0)
  const cutIndex = cutIndexes.length > 0 ? Math.min(...cutIndexes) : -1
  const path = cutIndex >= 0 ? trimmed.slice(0, cutIndex) : trimmed

  return path.startsWith("/") ? path : `/${path}`
}
