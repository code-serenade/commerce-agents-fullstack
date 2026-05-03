import type { ReactNode } from "react"
import {
  ButtonGroup as CoreButtonGroup,
  ButtonGroupSeparator as CoreButtonGroupSeparator,
  ButtonGroupText as CoreButtonGroupText,
} from "@workspace/ui-core/components/button-group"

export type ButtonGroupOrientation = "horizontal" | "vertical"

export interface ButtonGroupProps {
  items: ReactNode[]
  orientation?: ButtonGroupOrientation
}

export function ButtonGroup({
  items,
  orientation = "horizontal",
}: ButtonGroupProps) {
  return (
    <CoreButtonGroup orientation={orientation}>
      {items.map((item, index) => (
        <CoreButtonGroupText key={index}>
          {item}
          {index < items.length - 1 ? <CoreButtonGroupSeparator /> : null}
        </CoreButtonGroupText>
      ))}
    </CoreButtonGroup>
  )
}
