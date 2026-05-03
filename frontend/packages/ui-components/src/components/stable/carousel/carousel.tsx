import type { ReactNode } from "react"
import {
  Carousel as CoreCarousel,
  CarouselContent as CoreCarouselContent,
  CarouselItem as CoreCarouselItem,
  CarouselNext as CoreCarouselNext,
  CarouselPrevious as CoreCarouselPrevious,
} from "@workspace/ui-core/components/carousel"

export type CarouselOrientation = "horizontal" | "vertical"

export interface CarouselProps {
  slides: ReactNode[]
  orientation?: CarouselOrientation
}

export function Carousel({ slides, orientation = "horizontal" }: CarouselProps) {
  return (
    <CoreCarousel orientation={orientation}>
      <CoreCarouselContent>
        {slides.map((slide, index) => (
          <CoreCarouselItem key={index}>
            <div className="rounded-lg border p-6 text-center text-sm">{slide}</div>
          </CoreCarouselItem>
        ))}
      </CoreCarouselContent>
      <CoreCarouselPrevious />
      <CoreCarouselNext />
    </CoreCarousel>
  )
}
