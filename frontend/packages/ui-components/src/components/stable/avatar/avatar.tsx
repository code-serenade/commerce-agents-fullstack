import type { ReactNode } from "react"
import {
  Avatar as CoreAvatar,
  AvatarFallback as CoreAvatarFallback,
  AvatarGroup as CoreAvatarGroup,
  AvatarGroupCount as CoreAvatarGroupCount,
  AvatarImage as CoreAvatarImage,
} from "@workspace/ui-core/components/avatar"

export interface AvatarProps {
  src?: string
  alt: string
  fallback: ReactNode
}

export interface AvatarStackUser {
  src?: string
  alt: string
  fallback: ReactNode
}

export interface AvatarStackProps {
  users: AvatarStackUser[]
  overflowCount?: number
}

export function Avatar({ src, alt, fallback }: AvatarProps) {
  return (
    <CoreAvatar>
      {src ? <CoreAvatarImage src={src} alt={alt} /> : null}
      <CoreAvatarFallback>{fallback}</CoreAvatarFallback>
    </CoreAvatar>
  )
}

export function AvatarStack({ users, overflowCount = 0 }: AvatarStackProps) {
  return (
    <CoreAvatarGroup>
      {users.map((user, index) => (
        <Avatar
          key={`${user.alt}-${index}`}
          src={user.src}
          alt={user.alt}
          fallback={user.fallback}
        />
      ))}
      {overflowCount > 0 ? <CoreAvatarGroupCount>+{overflowCount}</CoreAvatarGroupCount> : null}
    </CoreAvatarGroup>
  )
}
