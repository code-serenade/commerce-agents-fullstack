import { useMutation } from "@tanstack/react-query"
import {
  streamFormDraftApi,
  type FormDraftStreamProcessEvent,
} from "@/api"
import { notifySuccess, type SimpleFormValues } from "../components/commerce-ui"
import type { CommerceTranslate } from "../components/commerce-types"
import { campaignFormDraftStreamPayload } from "./campaign-form-payloads"

export interface CampaignFormDraftStreamInput {
  onProcess?: (event: FormDraftStreamProcessEvent) => void
  values: SimpleFormValues
}

export function useCampaignFormDraftStream(t: CommerceTranslate) {
  return useMutation({
    mutationFn: ({ onProcess, values }: CampaignFormDraftStreamInput) =>
      streamFormDraftApi({
        onProcess,
        request: campaignFormDraftStreamPayload(values),
      }),
    onSuccess: () => {
      notifySuccess(t("commerce.campaigns.notify.draftFilled"))
    },
  })
}
