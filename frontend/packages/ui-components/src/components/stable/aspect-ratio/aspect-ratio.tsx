import { AspectRatio as CoreAspectRatio } from "@workspace/ui-core/components/aspect-ratio"

export interface AspectRatioProps {
  src: string
  alt: string
  ratio?: number
}

export function AspectRatio({ src, alt, ratio = 16 / 9 }: AspectRatioProps) {
  return (
    <CoreAspectRatio ratio={ratio}>
      <img src={src} alt={alt} className="h-full w-full rounded-lg object-cover" />
    </CoreAspectRatio>
  )
}
