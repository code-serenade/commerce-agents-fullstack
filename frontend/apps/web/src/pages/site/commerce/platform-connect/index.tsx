import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../components/commerce-ui"
import { useTranslation } from "react-i18next"
import { PlatformDialogs } from "./components/platform-dialogs"
import {
  PlatformConnectionsPanel,
  PlatformExecutionAdaptersPanel,
  PlatformMetrics,
  PlatformMetricsPanel,
  PlatformSupportPanel,
  PlatformSyncPanel,
} from "./components/platform-panels"
import { usePlatformConnectPage } from "./use-platform-connect-page"

export default function PlatformConnectPage() {
  const { t } = useTranslation()
  const page = usePlatformConnectPage()

  return (
    <CommerceSurface>
      <CommercePageHeader
        eyebrow={t("commerce.pages.platform.eyebrow")}
        title={t("commerce.pages.platform.title")}
        description={t("commerce.pages.platform.description")}
      />
      <CommerceBody>
        <PlatformMetrics items={page.metricCards} />
        <PlatformSupportPanel page={page} />
        <PlatformExecutionAdaptersPanel page={page} />
        <PlatformConnectionsPanel page={page} />
        <PlatformMetricsPanel page={page} />
        <PlatformSyncPanel page={page} />
      </CommerceBody>
      <PlatformDialogs page={page} />
    </CommerceSurface>
  )
}
