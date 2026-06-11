import { useState } from "react"
import { useTranslation } from "react-i18next"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import type { MetricCardsItem } from "@workspace/app-kit"
import {
  archiveCampaignApi,
  createCampaignFormDraftApi,
  createCampaignApi,
  getCampaignApi,
  listCampaignsApi,
  updateCampaignApi,
  type CampaignResponse,
  type FormDraftGenerationMode,
} from "@/api"
import {
  notifySuccess,
  toNumberOrNull,
  toOptionalString,
  type SimpleFormValues,
} from "../components/commerce-ui"
import { campaignFormDraftPayload } from "./campaign-form-payloads"
import { useCampaignFormDraftStream } from "./use-campaign-form-draft-stream"

const campaignsQueryKey = ["commerce", "campaigns"]

interface CampaignFormDraftInput {
  values: SimpleFormValues
  generationMode?: FormDraftGenerationMode
}

export function useCampaignsPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const [keyword, setKeyword] = useState("")
  const [status, setStatus] = useState("")
  const [dialog, setDialog] = useState<"create" | "edit" | null>(null)
  const [editingCampaign, setEditingCampaign] = useState<CampaignResponse | null>(null)

  const campaignsQuery = useQuery({
    queryKey: [...campaignsQueryKey, keyword, status],
    queryFn: () =>
      listCampaignsApi({
        keyword: keyword || undefined,
        pageSize: 100,
        status: status || undefined,
      }),
  })
  const campaigns = campaignsQuery.data?.campaigns ?? []
  const campaignDetailQuery = useQuery({
    queryKey: [...campaignsQueryKey, editingCampaign?.id, "detail"],
    enabled: Boolean(editingCampaign?.id),
    queryFn: () => getCampaignApi(editingCampaign!.id),
  })
  const editingCampaignDetail =
    campaignDetailQuery.data ?? editingCampaign
  const invalidateCampaigns = async () => {
    await queryClient.invalidateQueries({ queryKey: campaignsQueryKey })
  }
  const createCampaign = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createCampaignApi(campaignPayload(values)),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.campaigns.notify.created"))
      await invalidateCampaigns()
    },
  })
  const createCampaignFormDraft = useMutation({
    mutationFn: ({ values, generationMode }: CampaignFormDraftInput) =>
      createCampaignFormDraftApi(
        campaignFormDraftPayload(values, generationMode)
      ),
    onSuccess: () => {
      notifySuccess(t("commerce.campaigns.notify.draftFilled"))
    },
  })
  const createCampaignFormDraftStream = useCampaignFormDraftStream(t)
  const updateCampaign = useMutation({
    mutationFn: ({
      campaign,
      values,
    }: {
      campaign: CampaignResponse
      values: SimpleFormValues
    }) =>
      updateCampaignApi(campaign.id, {
        ...campaignPayload(values),
        actionMetadata: campaign.actionMetadata ?? {},
        metadata: campaign.metadata ?? {},
        output: campaign.output ?? {},
        status: values.status || campaign.status,
      }),
    onSuccess: async () => {
      setDialog(null)
      setEditingCampaign(null)
      notifySuccess(t("commerce.campaigns.notify.updated"))
      await invalidateCampaigns()
    },
  })
  const archiveCampaign = useMutation({
    mutationFn: archiveCampaignApi,
    onSuccess: async () => {
      notifySuccess(t("commerce.campaigns.notify.archived"))
      await invalidateCampaigns()
    },
  })

  return {
    archiveCampaign: archiveCampaign.mutate,
    campaigns,
    createCampaign: createCampaign.mutate,
    createCampaignFormDraft: createCampaignFormDraft.mutateAsync,
    createCampaignFormDraftPending:
      createCampaignFormDraft.isPending || createCampaignFormDraftStream.isPending,
    createCampaignFormDraftStream: createCampaignFormDraftStream.mutateAsync,
    createCampaignPending: createCampaign.isPending,
    dialog,
    editingCampaign: editingCampaignDetail,
    keyword,
    loading: campaignsQuery.isLoading || campaignDetailQuery.isLoading,
    metricCards: buildCampaignMetricCards(campaigns, t),
    setDialog: (nextDialog: "create" | "edit" | null) => {
      setDialog(nextDialog)
      if (!nextDialog) setEditingCampaign(null)
    },
    setEditingCampaign,
    setKeyword,
    setStatus,
    status,
    updateCampaign: updateCampaign.mutate,
  }
}

function campaignPayload(values: SimpleFormValues) {
  return {
    actionMetadata: {},
    budget: toNumberOrNull(values.budget),
    campaignType: values.campaignType || "promotion",
    description: toOptionalString(values.description),
    endDate: toOptionalString(values.endDate),
    metadata: {},
    name: values.name,
    output: {},
    productId: toOptionalString(values.productId),
    startDate: toOptionalString(values.startDate),
    status: values.status || "draft",
  }
}

function buildCampaignMetricCards(
  campaigns: CampaignResponse[],
  t: ReturnType<typeof useTranslation>["t"]
): MetricCardsItem[] {
  const active = campaigns.filter((item) => item.status === "active").length
  const draft = campaigns.filter((item) => item.status === "draft").length
  const paused = campaigns.filter((item) => item.status === "paused").length
  const budget = campaigns.reduce((sum, item) => sum + (item.budget ?? 0), 0)

  return [
    {
      key: "total",
      label: t("commerce.campaigns.metrics.total"),
      value: campaigns.length,
      tail: t("commerce.campaigns.metrics.totalTail"),
      variant: "accent",
    },
    {
      key: "active",
      label: t("commerce.campaigns.metrics.active"),
      value: active,
      tail: t("commerce.campaigns.metrics.activeTail", { draft, paused }),
      variant: "success",
    },
    {
      key: "budget",
      label: t("commerce.campaigns.metrics.budget"),
      value: budget,
      tail: t("commerce.campaigns.metrics.budgetTail"),
      variant: "default",
    },
    {
      key: "ended",
      label: t("commerce.campaigns.metrics.ended"),
      value: campaigns.filter((item) => item.status === "ended").length,
      tail: t("commerce.campaigns.metrics.endedTail"),
      variant: "warning",
    },
  ]
}

export type CampaignsPageState = ReturnType<typeof useCampaignsPage>
