import { Progress as CoreProgress } from "@workspace/ui-core/components/progress"

export interface ProgressProps {
  value: number
  max?: number
}

export function Progress({ value, max = 100 }: ProgressProps) {
  return <CoreProgress value={value} max={max} />
}
