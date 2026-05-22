import type { toRequestError } from "@workspace/services/errors/request-error"

export type LoginStage = "credentials" | "profile" | "workspace"

export function loginErrorCopyKey(
  error: ReturnType<typeof toRequestError>,
  stage: LoginStage
) {
  const status = error.status
  if (status === 400 || status === 401 || status === 403) {
    if (stage === "workspace") return "login.error.workspaceUnavailable"

    return "login.error.invalidCredentials"
  }

  if (error.kind === "network" || isServiceUnavailableStatus(status)) {
    return stage === "workspace"
      ? "login.error.backendUnavailable"
      : "login.error.authUnavailable"
  }

  return "login.error.workspaceUnavailable"
}

function isServiceUnavailableStatus(status?: number) {
  return status === 502 || status === 503 || status === 504
}
