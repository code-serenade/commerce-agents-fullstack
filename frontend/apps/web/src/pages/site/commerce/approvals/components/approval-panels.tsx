import { useState } from "react"
import { useTranslation } from "react-i18next"
import { MetricCards } from "@workspace/app-kit"
import { Button, NativeSelect } from "@workspace/ui-components"
import type { MetricCardsItem } from "@workspace/app-kit"
import type { AuditLogResponse, ToolApprovalResponse } from "@/api"
import { ApprovalExecutionOutcome } from "./approval-execution-outcome"
import {
  CommercePanel,
  CommerceTable,
  IdText,
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { ApprovalsPageState } from "../use-approvals-page"

export function ApprovalMetrics({ items }: { items: MetricCardsItem[] }) {
  return <MetricCards items={items} />
}

export function ApprovalRequestsPanel({ page }: { page: ApprovalsPageState }) {
  const { t } = useTranslation()
  const [statusDraft, setStatusDraft] = useState(page.approvalStatus)

  return (
    <CommercePanel
      title={t("commerce.approvals.panels.requests")}
      description={t("commerce.approvals.panels.requestsDesc")}
      actions={
        <Button size="sm" onClick={() => page.setDialog("approval")}>
          {t("commerce.approvals.panels.createRequest")}
        </Button>
      }
    >
      <div className="mb-3 w-full sm:max-w-52">
        <NativeSelect
          value={statusDraft}
          placeholder={t("commerce.common.status")}
          options={approvalStatusOptions(t)}
          onValueChange={(value) => {
            setStatusDraft(value)
            page.setApprovalStatus(value)
          }}
        />
      </div>
      <CommerceTable
        loading={page.loading}
        items={page.approvals}
        emptyText={t("commerce.approvals.panels.emptyRequests")}
        getRowId={(item) => item.id}
        columns={approvalColumns(page, t)}
      />
    </CommercePanel>
  )
}

export function AuditLogsPanel({ page }: { page: ApprovalsPageState }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("commerce.approvals.panels.audit")}
      description={t("commerce.approvals.panels.auditDesc")}
      actions={
        <Button size="sm" onClick={() => page.setDialog("audit")}>
          {t("commerce.approvals.panels.createAudit")}
        </Button>
      }
    >
      <CommerceTable
        loading={page.loading}
        items={page.auditLogs}
        emptyText={t("commerce.approvals.panels.emptyAudit")}
        getRowId={(item) => item.id}
        columns={auditColumns(t)}
      />
    </CommercePanel>
  )
}

function approvalColumns(
  page: ApprovalsPageState,
  t: CommerceTranslate
): CommerceColumn<ToolApprovalResponse>[] {
  return [
    {
      key: "skill",
      header: t("commerce.approvals.columns.skill"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.skillName}</div>
          <IdText>{item.id}</IdText>
        </div>
      ),
      minWidth: "14rem",
    },
    {
      key: "risk",
      header: t("commerce.approvals.columns.risk"),
      render: (item) => item.riskLevel || "L1",
      width: "6rem",
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>{item.status}</StatusBadge>
      ),
      width: "8rem",
    },
    {
      key: "reason",
      header: t("commerce.common.reason"),
      render: (item) => (
        <div className="max-w-96 space-y-1">
          <MutedText>{item.reason || "-"}</MutedText>
          <ApprovalReadiness result={item.result} />
        </div>
      ),
      minWidth: "18rem",
    },
    {
      key: "execution",
      header: t("commerce.approvals.columns.execution"),
      render: (item) => <ApprovalExecutionOutcome result={item.result} />,
      minWidth: "16rem",
    },
    {
      key: "actions",
      header: t("commerce.common.decision"),
      render: (item) => (
        <div className="flex flex-nowrap justify-end gap-2">
          <Button
            size="xs"
            variant="outline"
            disabled={item.status !== "pending"}
            onClick={() => page.approveApproval(item)}
          >
            {t("commerce.common.approve")}
          </Button>
          <Button
            size="xs"
            variant="outline"
            disabled={item.status !== "pending"}
            onClick={() => page.rejectApproval(item)}
          >
            {t("commerce.common.reject")}
          </Button>
        </div>
      ),
      sticky: "right",
      width: "9rem",
    },
  ]
}

function ApprovalReadiness({
  result,
}: {
  result?: ToolApprovalResponse["result"]
}) {
  const readiness = getRecord(result, "executionReadiness")
  const adapterStatus = getString(readiness, "adapterStatus")
  const message = getString(readiness, "message")

  if (!adapterStatus && !message) return null

  return (
    <div className="space-y-1">
      {adapterStatus ? (
        <StatusBadge tone={toneForStatus(adapterStatus)}>{adapterStatus}</StatusBadge>
      ) : null}
      {message ? <MutedText>{message}</MutedText> : null}
    </div>
  )
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

function auditColumns(t: CommerceTranslate): CommerceColumn<AuditLogResponse>[] {
  return [
    {
      key: "action",
      header: t("commerce.fields.action"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.action}</div>
          <MutedText>{item.targetType} / {item.targetId || "-"}</MutedText>
        </div>
      ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status || "success")}>
          {item.status || "success"}
        </StatusBadge>
      ),
    },
    {
      key: "reason",
      header: t("commerce.common.reason"),
      render: (item) => <MutedText>{item.reason || "-"}</MutedText>,
    },
  ]
}

function approvalStatusOptions(t: CommerceTranslate) {
  return [
    { label: t("commerce.common.all"), value: "" },
    { label: String("pending"), value: "pending" },
    { label: String("approved"), value: "approved" },
    { label: String("executed"), value: "executed" },
    { label: String("failed"), value: "failed" },
    { label: String("rejected"), value: "rejected" },
    { label: String("cancelled"), value: "cancelled" },
  ]
}
