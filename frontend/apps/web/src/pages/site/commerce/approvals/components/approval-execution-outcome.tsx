import { useTranslation } from "react-i18next"
import type { ToolApprovalResponse } from "@/api"
import {
  IdText,
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import {
  getCommerceEnumLabel,
  getCommerceStatusLabel,
} from "../../domain/enum-mappings"

export function ApprovalExecutionOutcome({
  result,
}: {
  result?: ToolApprovalResponse["result"]
}) {
  const { t } = useTranslation()
  const adapterStatus = getString(result, "adapterStatus")
  const executionStatus = getString(result, "executionStatus")
  const retryable = getBoolean(result, "retryable")
  const providerResult = getValue(result, "providerResult")
  const adapterRequest = getRecord(result, "adapterRequest")
  const idempotencyKey = getString(adapterRequest, "idempotencyKey")

  if (!adapterStatus && !executionStatus && retryable === undefined && !idempotencyKey) {
    return <MutedText>-</MutedText>
  }

  return (
    <div className="max-w-80 space-y-1">
      <div className="flex flex-wrap gap-1.5">
        {executionStatus ? (
          <StatusBadge tone={toneForStatus(executionStatus)}>
            {getCommerceStatusLabel(t, executionStatus)}
          </StatusBadge>
        ) : null}
        {adapterStatus ? (
          <StatusBadge tone={toneForStatus(adapterStatus)}>
            {getCommerceEnumLabel(t, "adapterStatus", adapterStatus)}
          </StatusBadge>
        ) : null}
      </div>
      {retryable !== undefined ? (
        <MutedText>
          {t("commerce.approvals.execution.retryable")}:{" "}
          {retryable
            ? t("commerce.approvals.execution.yes")
            : t("commerce.approvals.execution.no")}
        </MutedText>
      ) : null}
      <MutedText>
        {t("commerce.approvals.execution.providerResult")}:{" "}
        {formatProviderResult(providerResult, t)}
      </MutedText>
      {idempotencyKey ? <IdText>{idempotencyKey}</IdText> : null}
    </div>
  )
}

function formatProviderResult(
  value: unknown,
  t: ReturnType<typeof useTranslation>["t"]
) {
  if (value === null) return t("commerce.approvals.execution.nullValue")
  if (value && typeof value === "object") {
    return t("commerce.approvals.execution.objectValue")
  }
  return t("commerce.common.none")
}

function getValue(
  value: Record<string, unknown> | undefined,
  key: string
): unknown {
  return value?.[key]
}

function getBoolean(
  value: Record<string, unknown> | undefined,
  key: string
): boolean | undefined {
  const candidate = value?.[key]
  return typeof candidate === "boolean" ? candidate : undefined
}

function getRecord(
  value: unknown,
  key: string
): Record<string, unknown> | undefined {
  if (!value || typeof value !== "object" || Array.isArray(value)) return undefined
  const nested = (value as Record<string, unknown>)[key]
  return nested && typeof nested === "object" && !Array.isArray(nested)
    ? (nested as Record<string, unknown>)
    : undefined
}

function getString(
  value: Record<string, unknown> | undefined,
  key: string
): string | undefined {
  const candidate = value?.[key]
  return typeof candidate === "string" && candidate.trim()
    ? candidate
    : undefined
}
