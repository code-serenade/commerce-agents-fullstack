import { useState, type ReactNode } from "react"
import { FileText } from "lucide-react"
import { useTranslation } from "react-i18next"
import {
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  NativeSelect,
  Switch,
} from "@workspace/ui-components"
import type { AgentOutputBackendState } from "../use-agent-output-page"
import {
  defaultConversationReportDraft,
  getConversationReportScopeOptions,
  getConversationReportTypeOptions,
  type ConversationReportDraft,
} from "../view-model/conversation-report-view-model"

export function ConversationReportDialog({
  backend,
}: {
  backend: AgentOutputBackendState
}) {
  const { t } = useTranslation()
  const [draft, setDraft] = useState<ConversationReportDraft>(
    defaultConversationReportDraft
  )

  const updateDraft = <TKey extends keyof ConversationReportDraft>(
    key: TKey,
    value: ConversationReportDraft[TKey]
  ) => {
    setDraft((current) => ({
      ...current,
      [key]: value,
    }))
  }

  return (
    <Dialog
      open={backend.isReportDialogOpen}
      onOpenChange={(open) => {
        if (!open) backend.closeReportDialog()
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {t("agentOutput.backend.reportDialog.title")}
          </DialogTitle>
        </DialogHeader>

        <div className="grid gap-4 py-2">
          <Field label={t("agentOutput.backend.reportDialog.fields.reportType")}>
            <NativeSelect
              value={draft.reportType}
              options={getConversationReportTypeOptions(t)}
              disabled={backend.isGeneratingReport}
              onValueChange={(value) =>
                updateDraft(
                  "reportType",
                  value as ConversationReportDraft["reportType"]
                )
              }
            />
          </Field>
          <Field label={t("agentOutput.backend.reportDialog.fields.scope")}>
            <NativeSelect
              value={draft.scope}
              options={getConversationReportScopeOptions(t)}
              disabled={backend.isGeneratingReport}
              onValueChange={(value) =>
                updateDraft("scope", value as ConversationReportDraft["scope"])
              }
            />
          </Field>
          <SwitchField
            checked={draft.includeProcess}
            disabled={backend.isGeneratingReport}
            label={t("agentOutput.backend.reportDialog.fields.includeProcess")}
            onCheckedChange={(value) => updateDraft("includeProcess", value)}
          />
          <SwitchField
            checked={draft.includeSources}
            disabled={backend.isGeneratingReport}
            label={t("agentOutput.backend.reportDialog.fields.includeSources")}
            onCheckedChange={(value) => updateDraft("includeSources", value)}
          />
          <SwitchField
            checked={draft.includeTranscriptAppendix}
            disabled={backend.isGeneratingReport}
            label={t(
              "agentOutput.backend.reportDialog.fields.includeTranscriptAppendix"
            )}
            onCheckedChange={(value) =>
              updateDraft("includeTranscriptAppendix", value)
            }
          />
        </div>

        <DialogFooter>
          <Button
            type="button"
            variant="outline"
            disabled={backend.isGeneratingReport}
            onClick={() => {
              setDraft(defaultConversationReportDraft)
              backend.closeReportDialog()
            }}
          >
            {t("agentOutput.backend.reportDialog.cancel")}
          </Button>
          <Button
            type="button"
            disabled={backend.isGeneratingReport || !backend.canGenerateReport}
            onClick={() => {
              backend.generateReport(draft)
              setDraft(defaultConversationReportDraft)
            }}
          >
            <FileText className="size-4" />
            {backend.isGeneratingReport
              ? t("agentOutput.backend.reportDialog.generating")
              : t("agentOutput.backend.reportDialog.submit")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function Field({
  children,
  label,
}: {
  children: ReactNode
  label: string
}) {
  return (
    <label className="grid gap-1.5 text-sm font-medium text-foreground">
      <span>{label}</span>
      {children}
    </label>
  )
}

function SwitchField({
  checked,
  disabled,
  label,
  onCheckedChange,
}: {
  checked: boolean
  disabled: boolean
  label: string
  onCheckedChange: (checked: boolean) => void
}) {
  return (
    <label className="flex items-center justify-between gap-3 rounded-lg border border-(--app-border) px-3 py-2 text-sm text-foreground">
      <span className="min-w-0">{label}</span>
      <Switch
        checked={checked}
        disabled={disabled}
        size="sm"
        onCheckedChange={onCheckedChange}
      />
    </label>
  )
}
