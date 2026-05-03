import i18n from "@workspace/locales/i18n"
import type { RequestError } from "./request-error"

export function getRequestErrorMessage(error: RequestError): string {
  if (typeof error.code === "number") {
    const key = String(error.code)
    if (i18n.exists(key, { ns: "errors" })) {
      return i18n.t(key, { ns: "errors" })
    }
  }

  return i18n.t("default", { ns: "errors" })
}
