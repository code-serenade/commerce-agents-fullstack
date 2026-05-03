import type { ComponentType, SVGProps } from "react"

import type { UiCoreIconSlotName, UiCoreIconSlotProps } from "../icon-slots"

function BaseIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg
      aria-hidden="true"
      focusable="false"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      {...props}
    />
  )
}

export function DefaultCheckIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <path d="M20 6 9 17l-5-5" />
    </BaseIcon>
  )
}

export function DefaultChevronDownIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <path d="m6 9 6 6 6-6" />
    </BaseIcon>
  )
}

export function DefaultChevronUpIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <path d="m18 15-6-6-6 6" />
    </BaseIcon>
  )
}

export function DefaultChevronRightIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <path d="m9 18 6-6-6-6" />
    </BaseIcon>
  )
}

export function DefaultChevronLeftIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <path d="m15 18-6-6 6-6" />
    </BaseIcon>
  )
}

export function DefaultChevronsLeftIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <path d="m17 18-6-6 6-6" />
      <path d="m11 18-6-6 6-6" />
    </BaseIcon>
  )
}

export function DefaultChevronsRightIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <path d="m7 18 6-6-6-6" />
      <path d="m13 18 6-6-6-6" />
    </BaseIcon>
  )
}

export function DefaultMoreHorizontalIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <circle cx="12" cy="12" r="2" fill="currentColor" stroke="none" />
      <circle cx="18.5" cy="12" r="2" fill="currentColor" stroke="none" />
      <circle cx="5.5" cy="12" r="2" fill="currentColor" stroke="none" />
    </BaseIcon>
  )
}

export function DefaultXIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <path d="M18 6 6 18" />
      <path d="m6 6 12 12" />
    </BaseIcon>
  )
}

export function DefaultSearchIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <circle cx="11" cy="11" r="8" />
      <path d="m21 21-4.3-4.3" />
    </BaseIcon>
  )
}

export function DefaultMinusIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <path d="M5 12h14" />
    </BaseIcon>
  )
}

export function DefaultLoader2Icon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <path d="M21 12a9 9 0 1 1-6.2-8.6" />
    </BaseIcon>
  )
}

export function DefaultPanelLeftIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <path d="M9 4v16" />
    </BaseIcon>
  )
}

export function DefaultCircleCheckIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <circle cx="12" cy="12" r="10" />
      <path d="m9 12 2 2 4-4" />
    </BaseIcon>
  )
}

export function DefaultInfoIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <circle cx="12" cy="12" r="10" />
      <path d="M12 16v-4" />
      <path d="M12 8h.01" />
    </BaseIcon>
  )
}

export function DefaultTriangleAlertIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z" />
      <path d="M12 9v4" />
      <path d="M12 17h.01" />
    </BaseIcon>
  )
}

export function DefaultOctagonXIcon(props: UiCoreIconSlotProps) {
  return (
    <BaseIcon {...props}>
      <path d="m7.9 2.7 8.2.1 5.1 5.1-.1 8.2-5.1 5.1-8.2-.1-5.1-5.1.1-8.2 5.1-5.1Z" />
      <path d="m15 9-6 6" />
      <path d="m9 9 6 6" />
    </BaseIcon>
  )
}

export const defaultIconSlots: Record<
  UiCoreIconSlotName,
  ComponentType<UiCoreIconSlotProps>
> = {
  check: DefaultCheckIcon,
  "chevron-down": DefaultChevronDownIcon,
  "chevron-up": DefaultChevronUpIcon,
  "chevron-right": DefaultChevronRightIcon,
  "chevron-left": DefaultChevronLeftIcon,
  "chevrons-left": DefaultChevronsLeftIcon,
  "chevrons-right": DefaultChevronsRightIcon,
  "more-horizontal": DefaultMoreHorizontalIcon,
  x: DefaultXIcon,
  search: DefaultSearchIcon,
  minus: DefaultMinusIcon,
  "loader-2": DefaultLoader2Icon,
  "panel-left": DefaultPanelLeftIcon,
  "circle-check": DefaultCircleCheckIcon,
  info: DefaultInfoIcon,
  "triangle-alert": DefaultTriangleAlertIcon,
  "octagon-x": DefaultOctagonXIcon,
}
