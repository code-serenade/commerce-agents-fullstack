import { useTranslation } from "react-i18next"
import { StatusBadge, type StatusTone } from "../../components/commerce-ui"
import { fileReadStatusLabel } from "../../domain/enum-mappings"

const STATE_TONES = {
  failed: "danger",
  pending: "warning",
  readable: "success",
  unsupported: "muted",
} as const satisfies Record<string, StatusTone>

export function FilesReadableStatus({
  state,
}: {
  state: keyof typeof STATE_TONES
}) {
  const { t } = useTranslation()

  return (
    <StatusBadge tone={STATE_TONES[state]}>
      {fileReadStatusLabel(t, state)}
    </StatusBadge>
  )
}
