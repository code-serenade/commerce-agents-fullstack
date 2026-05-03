import type { ReactNode } from "react"
import {
  Field as CoreField,
  FieldContent as CoreFieldContent,
  FieldDescription as CoreFieldDescription,
  FieldError as CoreFieldError,
  FieldGroup as CoreFieldGroup,
  FieldLabel as CoreFieldLabel,
  FieldLegend as CoreFieldLegend,
  FieldSeparator as CoreFieldSeparator,
  FieldSet as CoreFieldSet,
  FieldTitle as CoreFieldTitle,
} from "@workspace/ui-core/components/field"

export interface FieldProps {
  children: ReactNode
}

export interface FieldSetProps {
  children: ReactNode
}

export interface FieldLegendProps {
  children: ReactNode
}

export interface FieldGroupProps {
  children: ReactNode
}

export interface FieldContentProps {
  children: ReactNode
}

export interface FieldLabelProps {
  children: ReactNode
}

export interface FieldTitleProps {
  children: ReactNode
}

export interface FieldDescriptionProps {
  children: ReactNode
}

export interface FieldSeparatorProps {}

export interface FieldErrorProps {
  children?: ReactNode
}

export function Field({ children }: FieldProps) {
  return <CoreField>{children}</CoreField>
}

export function FieldSet({ children }: FieldSetProps) {
  return <CoreFieldSet>{children}</CoreFieldSet>
}

export function FieldLegend({ children }: FieldLegendProps) {
  return <CoreFieldLegend>{children}</CoreFieldLegend>
}

export function FieldGroup({ children }: FieldGroupProps) {
  return <CoreFieldGroup>{children}</CoreFieldGroup>
}

export function FieldContent({ children }: FieldContentProps) {
  return <CoreFieldContent>{children}</CoreFieldContent>
}

export function FieldLabel({ children }: FieldLabelProps) {
  return <CoreFieldLabel>{children}</CoreFieldLabel>
}

export function FieldTitle({ children }: FieldTitleProps) {
  return <CoreFieldTitle>{children}</CoreFieldTitle>
}

export function FieldDescription({ children }: FieldDescriptionProps) {
  return <CoreFieldDescription>{children}</CoreFieldDescription>
}

export function FieldSeparator(_: FieldSeparatorProps) {
  return <CoreFieldSeparator />
}

export function FieldError({ children }: FieldErrorProps) {
  return <CoreFieldError>{children}</CoreFieldError>
}
