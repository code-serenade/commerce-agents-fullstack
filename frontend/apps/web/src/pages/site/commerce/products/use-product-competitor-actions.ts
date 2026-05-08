import { useMutation } from "@tanstack/react-query"
import {
  archiveProductCompetitorApi,
  createCompetitorChangeApi,
  createProductCompetitorApi,
  updateProductCompetitorApi,
  type ProductCompetitorResponse,
} from "@/api"
import {
  notifySuccess,
  toNumberOrNull,
  toOptionalString,
  type SimpleFormValues,
} from "../components/commerce-ui"
import type { CommerceTranslate } from "../components/commerce-types"
import type { ProductDialogName } from "./use-products-page"

export function useProductCompetitorActions({
  invalidateSelectedProduct,
  productId,
  selectedCompetitorId,
  setDialog,
  setSelectedCompetitorId,
  t,
}: {
  invalidateSelectedProduct: () => Promise<void>
  productId: string | undefined
  selectedCompetitorId: string | undefined
  setDialog: (dialog: ProductDialogName | null) => void
  setSelectedCompetitorId: (id: string | null) => void
  t: CommerceTranslate
}) {
  const createCompetitor = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createProductCompetitorApi(productId!, {
        metadata: {},
        monthlySales: toOptionalString(values.monthlySales),
        name: values.name,
        notes: toOptionalString(values.notes),
        platform: toOptionalString(values.platform),
        price: toNumberOrNull(values.price),
        rating: toNumberOrNull(values.rating),
        searchResults: {},
        url: toOptionalString(values.url),
      }),
    onSuccess: async (competitor) => {
      setSelectedCompetitorId(competitor.id)
      setDialog(null)
      notifySuccess(t("commerce.products.notify.competitorCreated"))
      await invalidateSelectedProduct()
    },
  })
  const archiveCompetitor = useMutation({
    mutationFn: (competitorId: string) =>
      archiveProductCompetitorApi(productId!, competitorId),
    onSuccess: async () => {
      setSelectedCompetitorId(null)
      notifySuccess(t("commerce.products.notify.competitorArchived"))
      await invalidateSelectedProduct()
    },
  })
  const updateCompetitor = useMutation({
    mutationFn: ({
      target,
      values,
    }: {
      target: ProductCompetitorResponse
      values: SimpleFormValues
    }) =>
      updateProductCompetitorApi(productId!, target.id, {
        metadata: target.metadata ?? {},
        monthlySales: toOptionalString(values.monthlySales),
        name: values.name,
        notes: toOptionalString(values.notes),
        platform: toOptionalString(values.platform),
        price: toNumberOrNull(values.price),
        rating: toNumberOrNull(values.rating),
        searchResults: target.searchResults ?? {},
        url: toOptionalString(values.url),
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.products.notify.competitorUpdated"))
      await invalidateSelectedProduct()
    },
  })
  const createCompetitorChange = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createCompetitorChangeApi(productId!, selectedCompetitorId!, {
        changeType: values.changeType,
        newValue: toOptionalString(values.newValue),
        oldValue: toOptionalString(values.oldValue),
        sourceType: values.sourceType || "manual",
        summary: toOptionalString(values.summary),
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.products.notify.changeCreated"))
      await invalidateSelectedProduct()
    },
  })

  return {
    archiveCompetitor: archiveCompetitor.mutate,
    createCompetitor: createCompetitor.mutate,
    createCompetitorChange: createCompetitorChange.mutate,
    updateCompetitor: updateCompetitor.mutate,
  }
}
