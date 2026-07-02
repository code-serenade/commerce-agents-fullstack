import { useTranslation } from "react-i18next"
import { PageLoading } from "@workspace/app-kit"

export function RouteLoadingFallback() {
  const { t } = useTranslation()

  return <PageLoading label={t("web.loading.route")} />
}
