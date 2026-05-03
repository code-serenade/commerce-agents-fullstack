import { Separator as CoreSeparator } from "@workspace/ui-core/components/separator"

export interface SeparatorProps {
  orientation?: "horizontal" | "vertical"
  decorative?: boolean
}

export function Separator({
  orientation = "horizontal",
  decorative = true,
}: SeparatorProps) {
  return <CoreSeparator orientation={orientation} decorative={decorative} />
}
