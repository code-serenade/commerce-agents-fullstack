import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
  InlineAlert,
} from "../components/commerce-ui"
import { useTranslation } from "react-i18next"
import { ApprovalDialogs } from "./components/approval-dialogs"
import {
  ApprovalMetrics,
  ApprovalRequestsPanel,
  AuditLogsPanel,
} from "./components/approval-panels"
import { useApprovalsPage } from "./use-approvals-page"

export default function ApprovalsPage() {
  const { t } = useTranslation()
  const page = useApprovalsPage()

  return (
    <CommerceSurface>
      <CommercePageHeader
        eyebrow={t("commerce.pages.approvals.eyebrow")}
        title={t("commerce.pages.approvals.title")}
        description={t("commerce.pages.approvals.description")}
      />
      <CommerceBody>
        <InlineAlert
          variant="warning"
          title={t("commerce.pages.approvals.boundaryTitle")}
          description={t("commerce.pages.approvals.boundaryDescription")}
        />
        <ApprovalMetrics items={page.metricCards} />
        <div className="grid gap-4 xl:grid-cols-[minmax(0,1.15fr)_minmax(24rem,0.85fr)]">
          <ApprovalRequestsPanel page={page} />
          <AuditLogsPanel page={page} />
        </div>
      </CommerceBody>
      <ApprovalDialogs page={page} />
    </CommerceSurface>
  )
}
