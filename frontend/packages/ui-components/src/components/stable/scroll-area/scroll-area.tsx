import type { ReactNode } from "react"
import { ScrollArea as CoreScrollArea } from "@workspace/ui-core/components/scroll-area"

export interface ScrollAreaProps {
  items: ReactNode[]
  height?: number
}

export function ScrollArea({ items, height = 160 }: ScrollAreaProps) {
  return (
    <CoreScrollArea>
      <div style={{ height }} className="pr-4">
        <div className="space-y-2">
          {items.map((item, index) => (
            <div key={index} className="rounded border px-3 py-2 text-sm">
              {item}
            </div>
          ))}
        </div>
      </div>
    </CoreScrollArea>
  )
}
