import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../components/commerce-ui"
import { useTranslation } from "react-i18next"
import { OperationDialogs } from "./components/operation-dialogs"
import {
  AlertsPanel,
  DailyBriefingPanel,
  LongTasksPanel,
  NotificationsPanel,
  OperationMetrics,
} from "./components/operation-panels"
import { useOperationsPage } from "./use-operations-page"

export default function OperationsPage() {
  const { t } = useTranslation()
  const page = useOperationsPage()

  return (
    <CommerceSurface>
      <CommercePageHeader
        eyebrow={t("commerce.pages.operations.eyebrow")}
        title={t("commerce.pages.operations.title")}
        description={t("commerce.pages.operations.description")}
      />
      <CommerceBody>
        <OperationMetrics items={page.metricCards} />
        <div className="grid gap-4 xl:grid-cols-2">
          <LongTasksPanel page={page} />
          <DailyBriefingPanel page={page} />
          <NotificationsPanel page={page} />
          <AlertsPanel page={page} />
        </div>
      </CommerceBody>
      <OperationDialogs page={page} />
    </CommerceSurface>
  )
}
