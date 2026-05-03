import { Avatar as AvatarPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { avatarClassNames } from "./avatar.styles"
import type {
  AvatarBadgeProps,
  AvatarClassResolver,
  AvatarFallbackProps,
  AvatarGroupCountProps,
  AvatarGroupProps,
  AvatarImageProps,
  AvatarProps,
  AvatarSize,
} from "./avatar.types"

function resolveStyledAvatarClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: AvatarClassResolver
}) {
  if (classResolver) {
    return classResolver({
      defaultClassName,
      className,
    })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function Avatar({
  mode = DEFAULT_MODE,
  className,
  size = "default",
  classNameMode = "merge",
  classResolver,
  ...props
}: AvatarProps) {
  if (mode === "primitive") {
    return <AvatarPrimitive.Root className={className} {...props} />
  }

  const resolvedSize = (size ?? "default") as AvatarSize

  return (
    <AvatarPrimitive.Root
      data-slot="avatar"
      data-size={resolvedSize}
      className={resolveStyledAvatarClassName({
        className,
        defaultClassName: avatarClassNames.slot1,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function AvatarImage({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: AvatarImageProps) {
  if (mode === "primitive") {
    return <AvatarPrimitive.Image className={className} {...props} />
  }

  return (
    <AvatarPrimitive.Image
      data-slot="avatar-image"
      className={resolveStyledAvatarClassName({
        className,
        defaultClassName: avatarClassNames.slot2,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function AvatarFallback({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: AvatarFallbackProps) {
  if (mode === "primitive") {
    return <AvatarPrimitive.Fallback className={className} {...props} />
  }

  return (
    <AvatarPrimitive.Fallback
      data-slot="avatar-fallback"
      className={resolveStyledAvatarClassName({
        className,
        defaultClassName: avatarClassNames.slot3,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function AvatarBadge({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: AvatarBadgeProps) {
  if (mode === "primitive") {
    return <span className={className} {...props} />
  }

  return (
    <span
      data-slot="avatar-badge"
      className={resolveStyledAvatarClassName({
        className,
        defaultClassName: cn(
          avatarClassNames.slot4,
          avatarClassNames.slot5,
          avatarClassNames.slot6,
          avatarClassNames.slot7
        ),
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function AvatarGroup({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: AvatarGroupProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="avatar-group"
      className={resolveStyledAvatarClassName({
        className,
        defaultClassName: avatarClassNames.slot8,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function AvatarGroupCount({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: AvatarGroupCountProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="avatar-group-count"
      className={resolveStyledAvatarClassName({
        className,
        defaultClassName: avatarClassNames.slot9,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export {
  Avatar,
  AvatarImage,
  AvatarFallback,
  AvatarGroup,
  AvatarGroupCount,
  AvatarBadge,
}
