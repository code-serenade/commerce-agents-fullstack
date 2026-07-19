import { toast } from "@workspace/ui-components"

export function notifySuccess(message: string) {
  toast.success(message)
}

export function notifyError(message: string) {
  toast.error(message)
}

export function downloadTextFile({
  content,
  fileName,
  mimeType,
}: {
  content: string
  fileName: string
  mimeType: string
}) {
  const blob = new Blob([content], {
    type: mimeType || "text/plain;charset=utf-8",
  })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement("a")
  anchor.href = url
  anchor.download = fileName
  anchor.click()
  URL.revokeObjectURL(url)
}

export function toNumberOrNull(value?: string | null): number | null {
  if (!value?.trim()) return null
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : null
}

export function toOptionalString(value?: string | null): string | null {
  const trimmed = value?.trim() ?? ""
  return trimmed ? trimmed : null
}

export function todayDateString() {
  return new Date().toISOString().slice(0, 10)
}
