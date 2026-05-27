import { Badge } from "@workspace/ui-components"
import { fileKind } from "../domain/logic"
import type { TrainingFileItem } from "../types"

interface FilesKindBadgeProps {
  file: TrainingFileItem
  t: (key: string) => string
}

export function FilesKindBadge({ file, t }: FilesKindBadgeProps) {
  const kind = fileKind(file)

  return (
    <Badge variant="outline">
      {t(`commerce.files.filters.kind.${kind}`)}
    </Badge>
  )
}
