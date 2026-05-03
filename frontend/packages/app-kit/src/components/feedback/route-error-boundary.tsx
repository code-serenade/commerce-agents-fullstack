import { isRouteErrorResponse, useRouteError } from "react-router"
import { PageErrorState } from "./page-error-state"

export function RouteErrorBoundary() {
  const error = useRouteError()

  const message = (() => {
    if (isRouteErrorResponse(error)) {
      return `${error.status} ${error.statusText || "Route error"}`
    }
    if (error instanceof Error && error.message) {
      return error.message
    }
    return "Unexpected page error"
  })()

  return <PageErrorState message={message} />
}
