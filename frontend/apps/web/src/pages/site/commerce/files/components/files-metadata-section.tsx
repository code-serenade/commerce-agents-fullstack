import { CopyableText } from "@workspace/app-kit"
import { formatTrainingFileMetadata } from "../domain/metadata-json"
import type { TrainingFileItem } from "../types"
import { FilesDetailSection } from "./files-detail-fields"

interface FilesMetadataSectionProps {
  file: TrainingFileItem
  t: (key: string) => string
}

export function FilesMetadataSection({ file, t }: FilesMetadataSectionProps) {
  const metadataText = formatTrainingFileMetadata(file)

  return (
    <FilesDetailSection
      title={t("commerce.files.detailDialog.sections.metadata")}
    >
      <div className="grid gap-2">
        <div className="flex min-w-0 flex-wrap items-center justify-between gap-2">
          <p className="text-xs text-muted-foreground">
            {t("commerce.files.detailDialog.metadata.description")}
          </p>
          <CopyableText value={metadataText}>
            {t("commerce.files.detailDialog.metadata.copy")}
          </CopyableText>
        </div>
        <pre className="max-h-48 overflow-auto whitespace-pre-wrap break-all rounded-md bg-background p-3 font-mono text-xs/5 text-muted-foreground">
          {metadataText}
        </pre>
      </div>
    </FilesDetailSection>
  )
}
