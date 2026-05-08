import { useTranslation } from "react-i18next"
import {
  CommercePanel,
  CommerceTable,
} from "../../components/commerce-ui"
import type {
  CommerceOverviewCampaign,
  CommerceOverviewProduct,
  CommerceOverviewWorkspace,
} from "../use-commerce-overview-page"
import { LinkButton } from "./overview-shared"
import {
  campaignColumns,
  productColumns,
  workspaceColumns,
} from "./overview-table-columns"

export function OverviewPrimaryLists({
  campaigns,
  loading,
  products,
  workspaces,
}: {
  campaigns: CommerceOverviewCampaign[]
  loading: boolean
  products: CommerceOverviewProduct[]
  workspaces: CommerceOverviewWorkspace[]
}) {
  const { t } = useTranslation()

  return (
    <div className="grid gap-4 xl:grid-cols-3">
      <CommercePanel
        title={t("commerce.overview.panels.productContext")}
        description={t("commerce.overview.panels.productContextDesc")}
        actions={
          <LinkButton to="/products" label={t("commerce.overview.panels.enterProducts")} />
        }
      >
        <CommerceTable
          loading={loading}
          items={products.slice(0, 5)}
          emptyText={t("commerce.overview.empty.products")}
          getRowId={(item) => item.id}
          columns={productColumns(t)}
        />
      </CommercePanel>
      <CommercePanel
        title={t("commerce.overview.panels.campaigns")}
        description={t("commerce.overview.panels.campaignsDesc")}
        actions={
          <LinkButton to="/campaigns" label={t("commerce.overview.panels.enterCampaigns")} />
        }
      >
        <CommerceTable
          loading={loading}
          items={campaigns.slice(0, 5)}
          emptyText={t("commerce.overview.empty.campaigns")}
          getRowId={(item) => item.id}
          columns={campaignColumns(t)}
        />
      </CommercePanel>
      <CommercePanel
        title={t("commerce.overview.panels.workspaces")}
        description={t("commerce.overview.panels.workspacesDesc")}
        actions={
          <LinkButton to="/workspaces" label={t("commerce.overview.panels.enterWorkspaces")} />
        }
      >
        <CommerceTable
          loading={loading}
          items={workspaces.slice(0, 5)}
          emptyText={t("commerce.overview.empty.workspaces")}
          getRowId={(item) => item.id}
          columns={workspaceColumns(t)}
        />
      </CommercePanel>
    </div>
  )
}
