import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../components/commerce-ui"
import { Button } from "@workspace/ui-components"
import { CalendarRange, Plus } from "lucide-react"
import { useTranslation } from "react-i18next"
import { DocumentIntroCard } from "../components/document-section"
import { CampaignDialogs } from "./components/campaign-dialogs"
import { CampaignsPanel } from "./components/campaign-panels"
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
        <DocumentIntroCard
          title={t("commerce.campaigns.panel.title")}
          description={t("commerce.campaigns.panel.description")}
          icon={<CalendarRange className="size-4" />}
          actions={
            <Button
              size="lg"
              variant="primary"
              onClick={() => page.setDialog("create")}
            >
              <Plus className="size-4" />
              {t("commerce.campaigns.panel.create")}
            </Button>
          }
        />
        <CampaignsPanel page={page} />
      </CommerceBody>
      <CampaignDialogs page={page} />
    </CommerceSurface>
  )
}
