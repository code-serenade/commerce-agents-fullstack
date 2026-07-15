import { useState } from "react"
import { useTranslation } from "react-i18next"
import { Button, NativeSelect } from "@workspace/ui-components"
import type { CampaignResponse } from "@/api"
import {
  CommercePanel,
  CommerceTable,
  CommerceToolbar,
  IdText,
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { CampaignsPageState } from "../use-campaigns-page"

export function CampaignsPanel({ page }: { page: CampaignsPageState }) {
  const { t } = useTranslation()
  const [statusDraft, setStatusDraft] = useState(page.status)

  return (
    <CommercePanel>
      <div className="space-y-3">
        <CommerceToolbar
          keyword={page.keyword}
          keywordPlaceholder={t("commerce.campaigns.panel.search")}
          onKeywordChange={page.setKeyword}
          filters={
            <div className="flex w-full gap-2 sm:w-auto">
              <NativeSelect
                value={statusDraft}
                placeholder={t("commerce.common.status")}
                options={statusOptions(t)}
                onValueChange={(value) => {
                  setStatusDraft(value)
                  page.setStatus(value)
                }}
              />
            </div>
          }
        />
        <CommerceTable
          loading={page.loading}
          items={page.campaigns}
          emptyText={t("commerce.campaigns.panel.empty")}
          getRowId={(item) => item.id}
          columns={campaignColumns(page, t)}
        />
      </div>
    </CommercePanel>
  )
}

function campaignColumns(
  page: CampaignsPageState,
  t: CommerceTranslate
): CommerceColumn<CampaignResponse>[] {
  return [
    {
      key: "name",
      header: t("commerce.campaigns.columns.campaign"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.name}</div>
          <IdText>{item.id}</IdText>
        </div>
      ),
    },
    {
      key: "type",
      header: t("commerce.common.type"),
      render: (item) => item.campaignType || "-",
    },
    {
      key: "budget",
      header: t("commerce.campaigns.columns.budget"),
      render: (item) => item.budget ?? "-",
    },
    {
      key: "period",
      header: t("commerce.campaigns.columns.period"),
      render: (item) => (
        <MutedText>
          {item.startDate || "-"} / {item.endDate || "-"}
        </MutedText>
      ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>
          {item.status}
        </StatusBadge>
      ),
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      render: (item) => (
        <div className="flex flex-wrap gap-2">
          <Button
            size="xs"
            variant="outline"
            onClick={() => {
              page.setEditingCampaign(item)
              page.setDialog("edit")
            }}
          >
            {t("commerce.common.edit")}
          </Button>
          <Button
            size="xs"
            variant="outline"
            onClick={() => page.archiveCampaign(item.id)}
          >
            {t("commerce.common.archive")}
          </Button>
        </div>
      ),
    },
  ]
}

function statusOptions(t: CommerceTranslate) {
  return [
    { label: t("commerce.common.all"), value: "" },
    { label: t("commerce.campaigns.options.draft"), value: "draft" },
    { label: t("commerce.campaigns.options.active"), value: "active" },
    { label: t("commerce.campaigns.options.paused"), value: "paused" },
    { label: t("commerce.campaigns.options.ended"), value: "ended" },
    { label: t("commerce.campaigns.options.archived"), value: "archived" },
  ]
}
