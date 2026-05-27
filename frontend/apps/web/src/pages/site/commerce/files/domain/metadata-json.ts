import type { TrainingFileItem } from "../types"

export function formatTrainingFileMetadata(file: TrainingFileItem) {
  return JSON.stringify(file.metadata, null, 2) ?? "{}"
}
