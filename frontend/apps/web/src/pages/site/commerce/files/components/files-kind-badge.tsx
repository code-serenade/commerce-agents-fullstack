import { Badge } from "@workspace/ui-components"
import { fileKindLabel } from "../../domain/enum-mappings"
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
      {fileKindLabel(t, kind)}
    </Badge>
  )
}
