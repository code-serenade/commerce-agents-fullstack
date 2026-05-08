import type {
  CommerceOverviewCampaign,
  CommerceOverviewPlatform,
  CommerceOverviewProduct,
  CommerceOverviewWorkspace,
} from "../use-commerce-overview-page"
import {
  IdText,
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"

export function productColumns(
  t: CommerceTranslate
): CommerceColumn<CommerceOverviewProduct>[] {
  return [
    {
      key: "name",
      header: t("commerce.overview.columns.product"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.name}</div>
          <IdText>{item.id}</IdText>
        </div>
      ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>{item.status}</StatusBadge>
      ),
    },
  ]
}

export function campaignColumns(
  t: CommerceTranslate
): CommerceColumn<CommerceOverviewCampaign>[] {
  return [
    {
      key: "name",
      header: t("commerce.overview.columns.campaign"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.name}</div>
          <MutedText>{item.campaignType || t("commerce.common.none")}</MutedText>
        </div>
      ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>{item.status}</StatusBadge>
      ),
    },
  ]
}

export function workspaceColumns(
  t: CommerceTranslate
): CommerceColumn<CommerceOverviewWorkspace>[] {
  return [
    {
      key: "title",
      header: t("commerce.overview.columns.workspace"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.title}</div>
          <MutedText>{item.workspaceType || "general"}</MutedText>
        </div>
      ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>{item.status}</StatusBadge>
      ),
    },
  ]
}

export function platformColumns(
  t: CommerceTranslate
): CommerceColumn<CommerceOverviewPlatform>[] {
  return [
    {
      key: "platform",
      header: t("commerce.overview.columns.platform"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.display}</div>
          <MutedText>{item.platform}</MutedText>
        </div>
      ),
    },
    {
      key: "gmv",
      header: t("commerce.overview.columns.gmv30d"),
      render: (item) => item.gmv30d ?? "-",
    },
    {
      key: "roi",
      header: t("commerce.overview.columns.roi"),
      render: (item) => item.adRoi30d ?? "-",
    },
  ]
}

export function governanceColumns(
  t: CommerceTranslate
): CommerceColumn<{
  id: string
  name: string
  note: string
  status: string
  type: string
}>[] {
  return [
    {
      key: "name",
      header: t("commerce.overview.columns.item"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.name}</div>
          <MutedText>{item.type}</MutedText>
        </div>
      ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>{item.status}</StatusBadge>
      ),
    },
  ]
}
