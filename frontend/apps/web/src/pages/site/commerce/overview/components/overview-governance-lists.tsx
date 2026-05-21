import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import {
  CommercePanel,
  CommerceTable,
} from "../../components/commerce-ui"
import type {
  CommerceOverviewApproval,
  CommerceOverviewExecution,
  CommerceOverviewPlatform,
} from "../use-commerce-overview-page"
import { LinkButton } from "./overview-shared"
import { governanceColumns, platformColumns } from "./overview-table-columns"

export function OverviewGovernanceLists({
  approvals,
  executionQueue,
  loading,
  platforms,
}: {
  approvals: CommerceOverviewApproval[]
  executionQueue: CommerceOverviewExecution[]
  loading: boolean
  platforms: CommerceOverviewPlatform[]
}) {
  const { t } = useTranslation()
  const governanceItems = useMemo(
    () =>
      [
        ...executionQueue.map((item) => ({
          id: item.id,
          name: item.actionType,
          note: item.description,
          status: item.status,
          type: t("commerce.overview.itemTypes.execution"),
        })),
        ...approvals.map((item) => ({
          id: item.id,
          name: item.skillName,
          note: item.reason ?? "",
          status: item.status,
          type: t("commerce.overview.itemTypes.approval"),
        })),
      ].slice(0, 8),
    [approvals, executionQueue, t]
  )

  return (
    <div className="grid gap-4 xl:grid-cols-2">
      <CommercePanel
        title={t("commerce.overview.panels.platform")}
        description={t("commerce.overview.panels.platformDesc")}
        actions={
          <LinkButton
            to="/platform-connect"
            label={t("commerce.overview.panels.enterPlatform")}
          />
        }
      >
        <CommerceTable
          loading={loading}
          items={platforms.slice(0, 6)}
          emptyText={t("commerce.overview.empty.platforms")}
          getRowId={(item) => item.platform}
          columns={platformColumns(t)}
        />
      </CommercePanel>
      <CommercePanel
        title={t("commerce.overview.panels.governance")}
        description={t("commerce.overview.panels.governanceDesc")}
        actions={
          <LinkButton
            to="/operations"
            label={t("commerce.overview.panels.enterOperations")}
          />
        }
      >
        <CommerceTable
          loading={loading}
          items={governanceItems}
          emptyText={t("commerce.overview.empty.governance")}
          getRowId={(item) => item.id}
          columns={governanceColumns(t)}
        />
      </CommercePanel>
    </div>
  )
}
