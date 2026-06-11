import type {
  CampaignFormDraftRequest,
  CampaignFormDraftStreamRequest,
  FormDraftGenerationMode,
} from "@/api"
import {
  toNumberOrNull,
  toOptionalString,
  type SimpleFormValues,
} from "../components/commerce-ui"

export function campaignFormDraftPayload(
  values: SimpleFormValues,
  generationMode?: FormDraftGenerationMode
): CampaignFormDraftRequest {
  const payload: CampaignFormDraftRequest = {
    budget: toNumberOrNull(values.budget),
    campaignType: toOptionalString(values.campaignType),
    description: toOptionalString(values.description),
    endDate: toOptionalString(values.endDate),
    goal: toOptionalString(values.name),
    name: toOptionalString(values.name),
    productId: toOptionalString(values.productId),
    prompt: toOptionalString(values.description),
    startDate: toOptionalString(values.startDate),
  }

  if (generationMode) {
    payload.generationMode = generationMode
  }

  return payload
}

export function campaignFormDraftStreamPayload(
  values: SimpleFormValues
): CampaignFormDraftStreamRequest {
  return {
    campaign: campaignFormDraftPayload(values, "agent_llm"),
    formType: "campaign",
  }
}
