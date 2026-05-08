import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
  InlineAlert,
} from "../components/commerce-ui"
import { useTranslation } from "react-i18next"
import {
  OverviewGovernanceLists,
  OverviewMetrics,
  OverviewPrimaryLists,
} from "./components/overview-panels"
import { useCommerceOverviewPage } from "./use-commerce-overview-page"

export default function CommerceOverviewPage() {
  const { t } = useTranslation()
  const page = useCommerceOverviewPage()

  return (
    <CommerceSurface>
      <CommercePageHeader
        eyebrow={t("commerce.pages.overview.eyebrow")}
        title={t("commerce.pages.overview.title")}
        description={t("commerce.pages.overview.description")}
      />
      <CommerceBody>
        <InlineAlert
          variant="warning"
          title={t("commerce.pages.overview.boundaryTitle")}
          description={t("commerce.pages.overview.boundaryDescription")}
        />
        <OverviewMetrics items={page.metricCards} />
        <OverviewPrimaryLists
          campaigns={page.campaigns}
          loading={page.loading}
          products={page.products}
          workspaces={page.workspaces}
        />
        <OverviewGovernanceLists
          approvals={page.approvals}
          executionQueue={page.executionQueue}
          loading={page.loading}
          packages={page.packages}
          platforms={page.platforms}
        />
      </CommerceBody>
    </CommerceSurface>
  )
}
