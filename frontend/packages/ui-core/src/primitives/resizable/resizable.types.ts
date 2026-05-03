import * as ResizablePrimitive from "react-resizable-panels"

import type { BaseMode } from "../../lib/component-mode"

export type ResizableClassNameMode = "merge" | "replace"

export type ResizableClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type ResizablePanelGroupProps = ResizablePrimitive.GroupProps & {
  mode?: BaseMode
  classNameMode?: ResizableClassNameMode
  classResolver?: ResizableClassResolver
}

export type ResizablePanelProps = ResizablePrimitive.PanelProps & {
  mode?: BaseMode
}

export type ResizableHandleProps = ResizablePrimitive.SeparatorProps & {
  mode?: BaseMode
  withHandle?: boolean
  classNameMode?: ResizableClassNameMode
  classResolver?: ResizableClassResolver
  handleClassName?: string
  handleClassNameMode?: ResizableClassNameMode
  handleClassResolver?: ResizableClassResolver
}
