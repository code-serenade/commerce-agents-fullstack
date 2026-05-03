import * as React from "react"
import { OTPInput } from "input-otp"

import type { BaseMode } from "../../lib/component-mode"

export type InputOtpClassNameMode = "merge" | "replace"

export type InputOtpClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type InputOTPProps = React.ComponentProps<typeof OTPInput> & {
  mode?: BaseMode
  containerClassName?: string
  classNameMode?: InputOtpClassNameMode
  classResolver?: InputOtpClassResolver
  containerClassNameMode?: InputOtpClassNameMode
  containerClassResolver?: InputOtpClassResolver
}

export type InputOTPGroupProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: InputOtpClassNameMode
  classResolver?: InputOtpClassResolver
}

export type InputOTPSlotProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  index: number
  classNameMode?: InputOtpClassNameMode
  classResolver?: InputOtpClassResolver
  caretContainerClassName?: string
  caretContainerClassNameMode?: InputOtpClassNameMode
  caretContainerClassResolver?: InputOtpClassResolver
  caretClassName?: string
  caretClassNameMode?: InputOtpClassNameMode
  caretClassResolver?: InputOtpClassResolver
}

export type InputOTPSeparatorProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: InputOtpClassNameMode
  classResolver?: InputOtpClassResolver
  icon?: React.ReactNode
}
