import { forwardRef, useImperativeHandle, useState } from "react"
import { useTranslation } from "react-i18next"
import { Input, Select, Textarea } from "@workspace/ui-components"
import type { TeamUsageStatus } from "@/api"
import type { TeamUsageRow } from "../types"

export interface EditTeamQuotaDialogContentValue {
  questionLimit: number
  tokenLimit: number | null
  status: TeamUsageStatus
  remark: string | null
}

export interface EditTeamQuotaDialogContentHandle {
  getValue: () => EditTeamQuotaDialogContentValue
}

function parseNonNegativeInt(value: string, fallback: number) {
  const parsed = Number.parseInt(value.trim(), 10)
  if (!Number.isFinite(parsed) || parsed < 0) {
    return fallback
  }

  return parsed
}

export const EditTeamQuotaDialogContent =
  forwardRef<EditTeamQuotaDialogContentHandle, { row: TeamUsageRow }>(
    function EditTeamQuotaDialogContent({ row }, ref) {
      const { t } = useTranslation()
      const [questionLimit, setQuestionLimit] = useState(
        String(row.question_limit)
      )
      const [tokenLimit, setTokenLimit] = useState(
        row.token_limit === null ? "" : String(row.token_limit)
      )
      const [status, setStatus] = useState<TeamUsageStatus>(row.status)
      const [remark, setRemark] = useState(row.remark ?? "")

      useImperativeHandle(
        ref,
        () => ({
          getValue: () => ({
            questionLimit: parseNonNegativeInt(
              questionLimit,
              row.question_limit
            ),
            tokenLimit: tokenLimit.trim()
              ? parseNonNegativeInt(tokenLimit, row.token_limit ?? 0)
              : null,
            status,
            remark: remark.trim() || null,
          }),
        }),
        [questionLimit, remark, row.question_limit, row.token_limit, status, tokenLimit]
      )

      return (
        <div className="grid gap-4 py-2">
          <div className="grid gap-2 sm:grid-cols-2">
            <div className="grid gap-2">
              <span className="text-sm font-medium">
                {t("admin.usage.teamUsage.edit.fields.schoolName")}
              </span>
              <span className="text-sm text-foreground">{row.school_name}</span>
            </div>
            <div className="grid gap-2">
              <span className="text-sm font-medium">
                {t("admin.usage.teamUsage.edit.fields.teamName")}
              </span>
              <span className="text-sm text-foreground">{row.team_name}</span>
            </div>
          </div>

          <div className="grid gap-2 sm:grid-cols-2">
            <div className="grid gap-2">
              <span className="text-sm font-medium">
                {t("admin.usage.teamUsage.edit.fields.questionLimit")}
              </span>
              <Input
                type="number"
                value={questionLimit}
                onValueChange={setQuestionLimit}
              />
            </div>
            <div className="grid gap-2">
              <span className="text-sm font-medium">
                {t("admin.usage.teamUsage.edit.fields.tokenLimit")}
              </span>
              <Input
                placeholder={t(
                  "admin.usage.teamUsage.edit.tokenLimitPlaceholder"
                )}
                type="number"
                value={tokenLimit}
                onValueChange={setTokenLimit}
              />
            </div>
          </div>

          <div className="grid gap-2">
            <span className="text-sm font-medium">
              {t("admin.usage.teamUsage.edit.fields.status")}
            </span>
            <Select
              value={status}
              onValueChange={(value) => setStatus(value as TeamUsageStatus)}
              list={[
                {
                  label: t("admin.usage.teamUsage.table.status.enabled"),
                  value: "enabled",
                },
                {
                  label: t("admin.usage.teamUsage.table.status.disabled"),
                  value: "disabled",
                },
              ]}
            />
          </div>

          <div className="grid gap-2">
            <span className="text-sm font-medium">
              {t("admin.usage.teamUsage.edit.fields.remark")}
            </span>
            <Textarea
              value={remark}
              onValueChange={setRemark}
              placeholder={t("admin.usage.teamUsage.edit.remarkPlaceholder")}
            />
          </div>
        </div>
      )
    }
  )
