import { useMutation } from "@tanstack/react-query"
import {
  streamFormDraftApi,
  type FormDraftStreamProcessEvent,
} from "@/api"
import { notifySuccess, type SimpleFormValues } from "../components/commerce-ui"
import type { CommerceTranslate } from "../components/commerce-types"
import { productDraftStreamPayload } from "./product-form-payloads"

export interface ProductDraftStreamInput {
  onProcess?: (event: FormDraftStreamProcessEvent) => void
  values: SimpleFormValues
}

export function useProductDraftStream(t: CommerceTranslate) {
  return useMutation({
    mutationFn: ({ onProcess, values }: ProductDraftStreamInput) =>
      streamFormDraftApi({
        onProcess,
        request: productDraftStreamPayload(values),
      }),
    onSuccess: () => {
      notifySuccess(t("commerce.products.notify.draftFilled"))
    },
  })
}
