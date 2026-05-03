import * as React from "react"

import type { BaseMode } from "../../lib/component-mode"

export type CardClassNameMode = "merge" | "replace"

export type CardClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type CardSize = "default" | "sm"

export type CardProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  size?: CardSize
  classNameMode?: CardClassNameMode
  classResolver?: CardClassResolver
}

export type CardHeaderProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: CardClassNameMode
  classResolver?: CardClassResolver
}

export type CardTitleProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: CardClassNameMode
  classResolver?: CardClassResolver
}

export type CardDescriptionProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: CardClassNameMode
  classResolver?: CardClassResolver
}

export type CardActionProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: CardClassNameMode
  classResolver?: CardClassResolver
}

export type CardContentProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: CardClassNameMode
  classResolver?: CardClassResolver
}

export type CardFooterProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: CardClassNameMode
  classResolver?: CardClassResolver
}
