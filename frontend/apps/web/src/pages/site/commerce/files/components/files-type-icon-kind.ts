import { fileKind } from "../domain/logic"
import type { TrainingFileItem } from "../types"

export type FilesIconKind = "document" | "image" | "sheet" | "text"

export function filesIconKind(file: TrainingFileItem): FilesIconKind {
  const kind = fileKind(file)

  if (kind === "image" || kind === "sheet" || kind === "text") return kind
  return "document"
}
