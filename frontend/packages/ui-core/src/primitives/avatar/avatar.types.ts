import * as React from "react"
import { Avatar as AvatarPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type AvatarClassNameMode = "merge" | "replace"

export type AvatarClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type AvatarSize = "default" | "sm" | "lg"

export type AvatarProps = React.ComponentProps<typeof AvatarPrimitive.Root> & {
  mode?: BaseMode
  size?: AvatarSize
  classNameMode?: AvatarClassNameMode
  classResolver?: AvatarClassResolver
}

export type AvatarImageProps = React.ComponentProps<
  typeof AvatarPrimitive.Image
> & {
  mode?: BaseMode
  classNameMode?: AvatarClassNameMode
  classResolver?: AvatarClassResolver
}

export type AvatarFallbackProps = React.ComponentProps<
  typeof AvatarPrimitive.Fallback
> & {
  mode?: BaseMode
  classNameMode?: AvatarClassNameMode
  classResolver?: AvatarClassResolver
}

export type AvatarBadgeProps = React.ComponentProps<"span"> & {
  mode?: BaseMode
  classNameMode?: AvatarClassNameMode
  classResolver?: AvatarClassResolver
}

export type AvatarGroupProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: AvatarClassNameMode
  classResolver?: AvatarClassResolver
}

export type AvatarGroupCountProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: AvatarClassNameMode
  classResolver?: AvatarClassResolver
}
