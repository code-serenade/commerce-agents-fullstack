import type { ReactNode } from "react"
import {
  ResizableHandle as CoreResizableHandle,
  ResizablePanel as CoreResizablePanel,
  ResizablePanelGroup as CoreResizablePanelGroup,
} from "@workspace/ui-core/components/resizable"

export type ResizableDirection = "horizontal" | "vertical"

export interface ResizableProps {
  left: ReactNode
  right: ReactNode
  direction?: ResizableDirection
  defaultLeftSize?: number
  minLeftSize?: number
  minRightSize?: number
}

export function Resizable({
  left,
  right,
  direction = "horizontal",
  defaultLeftSize = 40,
  minLeftSize = 20,
  minRightSize = 20,
}: ResizableProps) {
  return (
    <CoreResizablePanelGroup orientation={direction}>
      <CoreResizablePanel defaultSize={defaultLeftSize} minSize={minLeftSize}>
        <div className="h-full rounded border p-3">{left}</div>
      </CoreResizablePanel>

      <CoreResizableHandle withHandle />

      <CoreResizablePanel minSize={minRightSize}>
        <div className="h-full rounded border p-3">{right}</div>
      </CoreResizablePanel>
    </CoreResizablePanelGroup>
  )
}
