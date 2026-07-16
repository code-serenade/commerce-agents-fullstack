import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../components/commerce-ui"
import { Button } from "@workspace/ui-components"
import { Plus } from "lucide-react"
import { useTranslation } from "react-i18next"
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
      <CommerceBody>
        <CampaignsPanel page={page} />
      </CommerceBody>
      <CampaignDialogs page={page} />
    </CommerceSurface>
  )
}
