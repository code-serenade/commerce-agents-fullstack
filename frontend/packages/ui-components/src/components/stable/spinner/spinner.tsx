import { Spinner as CoreSpinner } from "@workspace/ui-core/components/spinner"

export type SpinnerSize = "sm" | "default" | "lg"

const SPINNER_SIZE_CLASSNAME: Record<SpinnerSize, string> = {
  sm: "size-3",
  default: "size-4",
  lg: "size-5",
}

export interface SpinnerProps {
  size?: SpinnerSize
}

export function Spinner({ size = "default" }: SpinnerProps) {
  return <CoreSpinner className={SPINNER_SIZE_CLASSNAME[size]} />
}
