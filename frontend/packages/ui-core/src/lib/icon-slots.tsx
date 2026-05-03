import {
  createContext,
  useContext,
  useMemo,
  type ComponentType,
  type PropsWithChildren,
  type SVGProps,
} from "react"

import { defaultIconSlots } from "./icons/default-icons"

export type UiCoreIconSlotProps = SVGProps<SVGSVGElement>

export type UiCoreIconSlotName =
  | "check"
  | "chevron-down"
  | "chevron-up"
  | "chevron-right"
  | "chevron-left"
  | "chevrons-left"
  | "chevrons-right"
  | "more-horizontal"
  | "x"
  | "search"
  | "minus"
  | "loader-2"
  | "panel-left"
  | "circle-check"
  | "info"
  | "triangle-alert"
  | "octagon-x"

export type UiCoreIconSlots = Partial<
  Record<UiCoreIconSlotName, ComponentType<UiCoreIconSlotProps>>
>

const UiCoreIconSlotsContext = createContext<UiCoreIconSlots>({})

export function UiCoreIconSlotsProvider({
  slots,
  children,
}: PropsWithChildren<{ slots: UiCoreIconSlots }>) {
  const parent = useContext(UiCoreIconSlotsContext)
  const merged = useMemo(() => ({ ...parent, ...slots }), [parent, slots])

  return (
    <UiCoreIconSlotsContext.Provider value={merged}>
      {children}
    </UiCoreIconSlotsContext.Provider>
  )
}

function createSlot(name: UiCoreIconSlotName) {
  return function UiCoreIconSlot(props: UiCoreIconSlotProps) {
    const slots = useContext(UiCoreIconSlotsContext)
    const Comp = slots[name]

    if (Comp) {
      return <Comp {...props} />
    }

    const Fallback = defaultIconSlots[name]

    if (Fallback) {
      return <Fallback {...props} />
    }

    return <svg aria-hidden="true" focusable="false" {...props} />
  }
}

export const CheckIcon = createSlot("check")
export const ChevronDownIcon = createSlot("chevron-down")
export const ChevronUpIcon = createSlot("chevron-up")
export const ChevronRightIcon = createSlot("chevron-right")
export const ChevronLeftIcon = createSlot("chevron-left")
export const ChevronsLeftIcon = createSlot("chevrons-left")
export const ChevronsRightIcon = createSlot("chevrons-right")
export const MoreHorizontalIcon = createSlot("more-horizontal")
export const XIcon = createSlot("x")
export const SearchIcon = createSlot("search")
export const MinusIcon = createSlot("minus")
export const Loader2Icon = createSlot("loader-2")
export const PanelLeftIcon = createSlot("panel-left")
export const CircleCheckIcon = createSlot("circle-check")
export const InfoIcon = createSlot("info")
export const TriangleAlertIcon = createSlot("triangle-alert")
export const OctagonXIcon = createSlot("octagon-x")
