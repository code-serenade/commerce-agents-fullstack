import type { ReactNode } from "react"
import {
  Item as CoreItem,
  ItemActions as CoreItemActions,
  ItemContent as CoreItemContent,
  ItemDescription as CoreItemDescription,
  ItemFooter as CoreItemFooter,
  ItemHeader as CoreItemHeader,
  ItemMedia as CoreItemMedia,
  ItemTitle as CoreItemTitle,
} from "@workspace/ui-core/components/item"

export type ItemVariant = "default" | "outline" | "muted"
export type ItemSize = "default" | "sm" | "xs"

export interface ItemProps {
  title: ReactNode
  description?: ReactNode
  leading?: ReactNode
  trailing?: ReactNode
  footer?: ReactNode
  variant?: ItemVariant
  size?: ItemSize
}

export function Item({
  title,
  description,
  leading,
  trailing,
  footer,
  variant = "default",
  size = "default",
}: ItemProps) {
  return (
    <CoreItem variant={variant} size={size}>
      {leading ? <CoreItemMedia>{leading}</CoreItemMedia> : null}

      <CoreItemContent>
        <CoreItemHeader>
          <CoreItemTitle>{title}</CoreItemTitle>
          {trailing ? <CoreItemActions>{trailing}</CoreItemActions> : null}
        </CoreItemHeader>

        {description ? <CoreItemDescription>{description}</CoreItemDescription> : null}

        {footer ? <CoreItemFooter>{footer}</CoreItemFooter> : null}
      </CoreItemContent>
    </CoreItem>
  )
}
