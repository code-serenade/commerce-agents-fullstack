import { Fragment } from "react"
import {
  InputOTP as CoreInputOTP,
  InputOTPGroup as CoreInputOTPGroup,
  InputOTPSeparator as CoreInputOTPSeparator,
  InputOTPSlot as CoreInputOTPSlot,
} from "@workspace/ui-core/components/input-otp"

export interface InputOTPProps {
  value: string
  onValueChange: (value: string) => void
  length?: number
}

export function InputOTP({ value, onValueChange, length = 6 }: InputOTPProps) {
  const indices = Array.from({ length }, (_, index) => index)
  const midpoint = Math.ceil(length / 2)

  return (
    <CoreInputOTP maxLength={length} value={value} onChange={onValueChange}>
      <CoreInputOTPGroup>
        {indices.slice(0, midpoint).map((index) => (
          <CoreInputOTPSlot key={index} index={index} />
        ))}
      </CoreInputOTPGroup>
      {length > 1 ? <CoreInputOTPSeparator /> : null}
      <CoreInputOTPGroup>
        {indices.slice(midpoint).map((index) => (
          <Fragment key={index}>
            <CoreInputOTPSlot index={index} />
          </Fragment>
        ))}
      </CoreInputOTPGroup>
    </CoreInputOTP>
  )
}
