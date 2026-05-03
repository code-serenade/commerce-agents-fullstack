import { Skeleton as CoreSkeleton } from "@workspace/ui-core/components/skeleton"

export type SkeletonSize = "sm" | "default" | "lg"

const SKELETON_SIZE_CLASSNAME: Record<SkeletonSize, string> = {
  sm: "h-3 w-24",
  default: "h-4 w-32",
  lg: "h-5 w-40",
}

export interface SkeletonProps {
  size?: SkeletonSize
}

export function Skeleton({ size = "default" }: SkeletonProps) {
  return <CoreSkeleton className={SKELETON_SIZE_CLASSNAME[size]} />
}
