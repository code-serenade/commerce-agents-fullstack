"use client"

import * as React from "react"
import { OTPInput, OTPInputContext } from "input-otp"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { MinusIcon } from "../../lib/icon-slots"
import { inputOtpClassNames } from "./input-otp.styles"
import type {
  InputOtpClassResolver,
  InputOTPGroupProps,
  InputOTPProps,
  InputOTPSeparatorProps,
  InputOTPSlotProps,
} from "./input-otp.types"

function resolveStyledInputOtpClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: InputOtpClassResolver
}) {
  if (classResolver) {
    return classResolver({
      defaultClassName,
      className,
    })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function InputOTP({
  mode = DEFAULT_MODE,
  className,
  containerClassName,
  classNameMode = "merge",
  classResolver,
  containerClassNameMode = "merge",
  containerClassResolver,
  ...props
}: InputOTPProps) {
  if (mode === "primitive") {
    return (
      <OTPInput
        containerClassName={containerClassName}
        spellCheck={false}
        className={className}
        {...props}
      />
    )
  }

  return (
    <OTPInput
      data-slot="input-otp"
      containerClassName={resolveStyledInputOtpClassName({
        className: containerClassName,
        defaultClassName: inputOtpClassNames.slot4,
        classNameMode: containerClassNameMode,
        classResolver: containerClassResolver,
      })}
      spellCheck={false}
      className={resolveStyledInputOtpClassName({
        className,
        defaultClassName: inputOtpClassNames.slot0,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function InputOTPGroup({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: InputOTPGroupProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="input-otp-group"
      className={resolveStyledInputOtpClassName({
        className,
        defaultClassName: inputOtpClassNames.slot5,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function InputOTPSlot({
  mode = DEFAULT_MODE,
  index,
  className,
  classNameMode = "merge",
  classResolver,
  caretContainerClassName,
  caretContainerClassNameMode = "merge",
  caretContainerClassResolver,
  caretClassName,
  caretClassNameMode = "merge",
  caretClassResolver,
  ...props
}: InputOTPSlotProps) {
  const inputOTPContext = React.useContext(OTPInputContext)
  const { char, hasFakeCaret, isActive } = inputOTPContext?.slots[index] ?? {}

  if (mode === "primitive") {
    return (
      <div className={className} {...props}>
        {char}
        {hasFakeCaret && (
          <div className={caretContainerClassName}>
            <div className={caretClassName} />
          </div>
        )}
      </div>
    )
  }

  return (
    <div
      data-slot="input-otp-slot"
      data-active={isActive}
      className={resolveStyledInputOtpClassName({
        className,
        defaultClassName: inputOtpClassNames.slot6,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      {char}
      {hasFakeCaret && (
        <div
          className={resolveStyledInputOtpClassName({
            className: caretContainerClassName,
            defaultClassName: inputOtpClassNames.slot1,
            classNameMode: caretContainerClassNameMode,
            classResolver: caretContainerClassResolver,
          })}
        >
          <div
            className={resolveStyledInputOtpClassName({
              className: caretClassName,
              defaultClassName: inputOtpClassNames.slot2,
              classNameMode: caretClassNameMode,
              classResolver: caretClassResolver,
            })}
          />
        </div>
      )}
    </div>
  )
}

function InputOTPSeparator({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  icon,
  ...props
}: InputOTPSeparatorProps) {
  if (mode === "primitive") {
    return (
      <div className={className} role="separator" {...props}>
        {icon ?? <MinusIcon />}
      </div>
    )
  }

  return (
    <div
      data-slot="input-otp-separator"
      className={resolveStyledInputOtpClassName({
        className,
        defaultClassName: inputOtpClassNames.slot3,
        classNameMode,
        classResolver,
      })}
      role="separator"
      {...props}
    >
      {icon ?? <MinusIcon />}
    </div>
  )
}

export { InputOTP, InputOTPGroup, InputOTPSlot, InputOTPSeparator }
