import { fileUsageLabel } from "../../domain/enum-mappings"
import type { FileConversationUseState } from "../domain/logic"
import { FilesDetailRow, FilesDetailSection } from "./files-detail-fields"

interface FilesUseStateSectionProps {
  state: FileConversationUseState
  t: (key: string) => string
}

export function FilesUseStateSection({ state, t }: FilesUseStateSectionProps) {
  return (
    <FilesDetailSection title={t("commerce.files.detailDialog.sections.usage")}>
      <FilesDetailRow
        label={t("commerce.files.detailDialog.fields.usageState")}
        value={fileUsageLabel(t, state)}
      />
      <FilesDetailRow
        label={t("commerce.files.detailDialog.fields.aiReadableThisTurn")}
        value={
          state === "selectedForTurn"
            ? t("commerce.files.detailDialog.yes")
            : t("commerce.files.detailDialog.no")
        }
      />
      <p className="text-xs text-muted-foreground">
        {t(`commerce.files.detailDialog.usageHint.${state}`)}
      </p>
    </FilesDetailSection>
  )
}
