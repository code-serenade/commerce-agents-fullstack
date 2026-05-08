import { useState } from "react"
import { useTranslation } from "react-i18next"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import type { MetricCardsItem } from "@workspace/app-kit"
import {
  approveToolApprovalApi,
  createAuditLogApi,
  createToolApprovalApi,
  listAuditLogsApi,
  listToolApprovalsApi,
  rejectToolApprovalApi,
  type AuditLogResponse,
  type ToolApprovalResponse,
} from "@/api"
import {
  notifySuccess,
  toOptionalString,
  type SimpleFormValues,
} from "../components/commerce-ui"

const approvalsQueryKey = ["commerce", "approvals"]

export function useApprovalsPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const [approvalStatus, setApprovalStatus] = useState("")
  const [dialog, setDialog] = useState<"approval" | "audit" | null>(null)
  const approvalsQuery = useQuery({
    queryKey: [...approvalsQueryKey, "tool-requests", approvalStatus],
    queryFn: () =>
      listToolApprovalsApi({
        pageSize: 100,
        status: approvalStatus || undefined,
      }),
  })
  const auditLogsQuery = useQuery({
    queryKey: [...approvalsQueryKey, "audit-logs"],
    queryFn: () => listAuditLogsApi({ pageSize: 100 }),
  })
  const invalidate = async () => {
    await queryClient.invalidateQueries({ queryKey: approvalsQueryKey })
  }
  const createApproval = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createToolApprovalApi({
        actionKey: toOptionalString(values.actionKey),
        args: {},
        conversationId: toOptionalString(values.conversationId),
        reason: toOptionalString(values.reason),
        requestRole: toOptionalString(values.requestRole),
        riskLevel: values.riskLevel || "L1",
        runId: toOptionalString(values.runId),
        skillName: values.skillName,
        toolCallId: toOptionalString(values.toolCallId),
        workspaceId: toOptionalString(values.workspaceId),
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.approvals.notify.created"))
      await invalidate()
    },
  })
  const approveApproval = useMutation({
    mutationFn: (approval: ToolApprovalResponse) =>
      approveToolApprovalApi(approval.id, {
        reason: t("commerce.common.manualApprove"),
        result: {},
      }),
    onSuccess: async () => {
      notifySuccess(t("commerce.approvals.notify.approved"))
      await invalidate()
    },
  })
  const rejectApproval = useMutation({
    mutationFn: (approval: ToolApprovalResponse) =>
      rejectToolApprovalApi(approval.id, {
        reason: t("commerce.common.manualReject"),
        result: {},
      }),
    onSuccess: async () => {
      notifySuccess(t("commerce.approvals.notify.rejected"))
      await invalidate()
    },
  })
  const createAudit = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createAuditLogApi({
        action: values.action,
        executionQueueId: toOptionalString(values.executionQueueId),
        metadata: {},
        reason: toOptionalString(values.reason),
        requestId: toOptionalString(values.requestId),
        runId: toOptionalString(values.runId),
        status: values.status || "success",
        targetId: toOptionalString(values.targetId),
        targetType: values.targetType,
        toolApprovalId: toOptionalString(values.toolApprovalId),
        traceId: toOptionalString(values.traceId),
        workspaceId: toOptionalString(values.workspaceId),
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.approvals.notify.auditCreated"))
      await invalidate()
    },
  })
  const approvals = approvalsQuery.data?.items ?? []
  const auditLogs = auditLogsQuery.data?.logs ?? []

  return {
    approvalStatus,
    approvals,
    approveApproval: approveApproval.mutate,
    auditLogs,
    createApproval: createApproval.mutate,
    createAudit: createAudit.mutate,
    dialog,
    loading: approvalsQuery.isLoading || auditLogsQuery.isLoading,
    metricCards: buildApprovalMetricCards(approvals, auditLogs, t),
    rejectApproval: rejectApproval.mutate,
    setApprovalStatus,
    setDialog,
  }
}

function buildApprovalMetricCards(
  approvals: ToolApprovalResponse[],
  auditLogs: AuditLogResponse[],
  t: ReturnType<typeof useTranslation>["t"]
): MetricCardsItem[] {
  return [
    {
      key: "approvals",
      label: t("commerce.approvals.metrics.requests"),
      value: approvals.length,
      tail: t("commerce.common.countPending", {
        count: approvals.filter((item) => item.status === "pending").length,
      }),
      variant: "accent",
    },
    {
      key: "approved",
      label: t("commerce.approvals.metrics.approved"),
      value: approvals.filter((item) => item.status === "approved").length,
      tail: t("commerce.approvals.metrics.approvedTail"),
      variant: "success",
    },
    {
      key: "rejected",
      label: t("commerce.approvals.metrics.rejected"),
      value: approvals.filter((item) => item.status === "rejected").length,
      tail: t("commerce.approvals.metrics.rejectedTail"),
      variant: "warning",
    },
    {
      key: "audit",
      label: t("commerce.approvals.metrics.audit"),
      value: auditLogs.length,
      tail: t("commerce.approvals.metrics.auditTail"),
      variant: "default",
    },
  ]
}

export type ApprovalsPageState = ReturnType<typeof useApprovalsPage>
