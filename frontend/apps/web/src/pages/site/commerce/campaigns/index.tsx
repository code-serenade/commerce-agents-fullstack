import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../components/commerce-ui"
import { useTranslation } from "react-i18next"
import { CampaignDialogs } from "./components/campaign-dialogs"
import { CampaignMetrics, CampaignsPanel } from "./components/campaign-panels"
import { useCampaignsPage } from "./use-campaigns-page"

export default function CampaignsPage() {
  const { t } = useTranslation()
  const page = useCampaignsPage()

  return (
    <CommerceSurface>
      <CommercePageHeader
        eyebrow={t("commerce.pages.campaigns.eyebrow")}
        title={t("commerce.pages.campaigns.title")}
        description={t("commerce.pages.campaigns.description")}
      />
      <CommerceBody>
        <CampaignMetrics items={page.metricCards} />
        <CampaignsPanel page={page} />
      </CommerceBody>
      <CampaignDialogs page={page} />
    </CommerceSurface>
  )
}
