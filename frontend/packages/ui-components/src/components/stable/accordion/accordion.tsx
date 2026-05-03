import type { ReactNode } from "react"
import {
  Accordion as CoreAccordion,
  AccordionContent as CoreAccordionContent,
  AccordionItem as CoreAccordionItem,
  AccordionTrigger as CoreAccordionTrigger,
} from "@workspace/ui-core/components/accordion"

export type AccordionType = "single" | "multiple"

interface AccordionBaseProps {
  children: ReactNode
  disabled?: boolean
}

export interface AccordionSingleProps extends AccordionBaseProps {
  type?: "single"
  value?: string
  defaultValue?: string
  onValueChange?: (value: string) => void
  collapsible?: boolean
}

export interface AccordionMultipleProps extends AccordionBaseProps {
  type: "multiple"
  value?: string[]
  defaultValue?: string[]
  onValueChange?: (value: string[]) => void
}

export type AccordionProps = AccordionSingleProps | AccordionMultipleProps

export interface AccordionItemProps {
  value: string
  disabled?: boolean
  children: ReactNode
}

export interface AccordionTriggerProps {
  children: ReactNode
}

export interface AccordionContentProps {
  children: ReactNode
}

export function Accordion(props: AccordionProps) {
  if (props.type === "multiple") {
    const {
      type,
      value,
      defaultValue,
      onValueChange,
      disabled = false,
      children,
    } = props

    return (
      <CoreAccordion
        type={type}
        value={value}
        defaultValue={defaultValue}
        onValueChange={onValueChange}
        disabled={disabled}
      >
        {children}
      </CoreAccordion>
    )
  }

  const {
    type = "single",
    value,
    defaultValue,
    onValueChange,
    collapsible = false,
    disabled = false,
    children,
  } = props

  return (
    <CoreAccordion
      type={type}
      value={value}
      defaultValue={defaultValue}
      onValueChange={onValueChange}
      collapsible={collapsible}
      disabled={disabled}
    >
      {children}
    </CoreAccordion>
  )
}

export function AccordionItem({ value, disabled = false, children }: AccordionItemProps) {
  return (
    <CoreAccordionItem value={value} disabled={disabled}>
      {children}
    </CoreAccordionItem>
  )
}

export function AccordionTrigger({ children }: AccordionTriggerProps) {
  return <CoreAccordionTrigger>{children}</CoreAccordionTrigger>
}

export function AccordionContent({ children }: AccordionContentProps) {
  return <CoreAccordionContent>{children}</CoreAccordionContent>
}
