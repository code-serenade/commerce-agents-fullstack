import { useEffect, useMemo, useState } from "react"
import type { PointerEvent as ReactPointerEvent, ReactNode } from "react"
import {
  ChevronLeft,
  ChevronRight,
  Download,
  Eye,
  FlipHorizontal2,
  ImageOff,
  RotateCw,
  Scan,
  X,
  ZoomIn,
  ZoomOut,
} from "lucide-react"
import {
  Dialog as CoreDialog,
  DialogContent as CoreDialogContent,
  DialogTitle as CoreDialogTitle,
} from "@workspace/ui-core/components/dialog"

const MIN_SCALE = 0.25
const MAX_SCALE = 4
const SCALE_STEP = 0.25

export interface ImageViewerImage {
  src: string
  alt: string
  thumbnailSrc?: string
  downloadName?: string
  downloadable?: boolean
}

export interface ImageViewerLabels {
  preview: string
  close: string
  previous: string
  next: string
  mirror: string
  rotate: string
  zoomOut: string
  zoomIn: string
  reset: string
  download: string
  loadError: string
}

export type ImageViewerThumbnailSize = "compact" | "default"

export interface ImageViewerProps {
  images: readonly ImageViewerImage[]
  open: boolean
  index: number
  onOpenChange: (open: boolean) => void
  onIndexChange: (index: number) => void
  labels?: Partial<ImageViewerLabels>
  thumbnailSize?: ImageViewerThumbnailSize
}

const DEFAULT_LABELS: ImageViewerLabels = {
  preview: "Preview image",
  close: "Close image viewer",
  previous: "Previous image",
  next: "Next image",
  mirror: "Mirror image",
  rotate: "Rotate image",
  zoomOut: "Zoom out",
  zoomIn: "Zoom in",
  reset: "Reset image",
  download: "Download image",
  loadError: "Image could not be displayed",
}

function clampScale(value: number) {
  return Math.min(MAX_SCALE, Math.max(MIN_SCALE, value))
}

export function ImageViewer({
  images,
  open,
  index,
  onOpenChange,
  onIndexChange,
  labels,
  thumbnailSize = "default",
}: ImageViewerProps) {
  const copy = useMemo(() => ({ ...DEFAULT_LABELS, ...labels }), [labels])
  const safeIndex = images.length === 0 ? 0 : Math.min(Math.max(index, 0), images.length - 1)
  const activeImage = images[safeIndex]
  const [scale, setScale] = useState(1)
  const [rotation, setRotation] = useState(0)
  const [mirrored, setMirrored] = useState(false)
  const [offset, setOffset] = useState({ x: 0, y: 0 })
  const [dragStart, setDragStart] = useState<{ x: number; y: number } | null>(null)
  const [failedSources, setFailedSources] = useState<ReadonlySet<string>>(new Set())

  const resetTransform = () => {
    setScale(1)
    setRotation(0)
    setMirrored(false)
    setOffset({ x: 0, y: 0 })
    setDragStart(null)
  }

  useEffect(() => {
    resetTransform()
  }, [open, safeIndex])

  useEffect(() => {
    if (!open || images.length < 2) return

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "ArrowLeft") {
        event.preventDefault()
        onIndexChange((safeIndex - 1 + images.length) % images.length)
      }
      if (event.key === "ArrowRight") {
        event.preventDefault()
        onIndexChange((safeIndex + 1) % images.length)
      }
    }

    window.addEventListener("keydown", onKeyDown)
    return () => window.removeEventListener("keydown", onKeyDown)
  }, [images.length, onIndexChange, open, safeIndex])

  if (!activeImage) return null

  const moveTo = (nextIndex: number) => {
    onIndexChange((nextIndex + images.length) % images.length)
  }

  const zoomBy = (amount: number) => {
    setScale((value) => {
      const nextScale = clampScale(value + amount)
      if (nextScale === 1) setOffset({ x: 0, y: 0 })
      return nextScale
    })
  }

  const handlePointerDown = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (scale <= 1) return
    event.currentTarget.setPointerCapture(event.pointerId)
    setDragStart({ x: event.clientX - offset.x, y: event.clientY - offset.y })
  }

  const handlePointerMove = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (!dragStart) return
    setOffset({ x: event.clientX - dragStart.x, y: event.clientY - dragStart.y })
  }

  const handlePointerUp = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (!dragStart) return
    event.currentTarget.releasePointerCapture(event.pointerId)
    setDragStart(null)
  }

  const handleDownload = () => {
    const anchor = document.createElement("a")
    anchor.href = activeImage.src
    anchor.download = activeImage.downloadName ?? "image"
    anchor.rel = "noopener"
    anchor.click()
  }

  const handleImageError = () => {
    setFailedSources((sources) => new Set(sources).add(activeImage.src))
  }

  const imageFailed = failedSources.has(activeImage.src)

  return (
    <>
      <button
        type="button"
        className={
          thumbnailSize === "compact"
            ? "group relative flex size-12 shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-md border border-muted bg-background"
            : "group relative flex size-40 cursor-pointer items-center justify-center overflow-hidden rounded-md border-4 border-muted bg-background"
        }
        aria-label={copy.preview}
        onClick={() => onOpenChange(true)}
      >
        {imageFailed ? (
          <ImageOff aria-hidden="true" className="size-5 text-muted-foreground" />
        ) : (
          <img
            src={activeImage.thumbnailSrc ?? activeImage.src}
            alt={activeImage.alt}
            className="size-full object-cover"
            onError={handleImageError}
          />
        )}
        <span className="absolute inset-0 flex items-center justify-center gap-2 bg-black/60 text-sm font-medium text-white opacity-0 transition-opacity group-hover:opacity-100 group-focus-visible:opacity-100">
          <Eye aria-hidden="true" className="size-4" />
          <span className={thumbnailSize === "compact" ? "sr-only" : undefined}>
            {copy.preview}
          </span>
        </span>
      </button>

      <CoreDialog open={open} onOpenChange={onOpenChange}>
        <CoreDialogContent
          showCloseButton={false}
          className="inset-0 top-0 left-0 flex h-dvh w-screen max-w-none translate-x-0 translate-y-0 flex-col gap-0 overflow-hidden rounded-none border-0 bg-black/60 p-0 text-white ring-0 sm:max-w-none dark:bg-black/60"
          onOpenAutoFocus={(event) => event.preventDefault()}
        >
          <CoreDialogTitle className="sr-only">{activeImage.alt}</CoreDialogTitle>

          <div className="absolute top-4 left-4 z-20 flex max-w-[calc(100%-5rem)] translate-x-0 items-center overflow-x-auto rounded-md bg-black/65 p-1 shadow-lg backdrop-blur-sm sm:left-1/2 sm:-translate-x-1/2">
            <ViewerButton label={copy.mirror} onClick={() => setMirrored((value) => !value)}>
              <FlipHorizontal2 />
            </ViewerButton>
            <ViewerButton label={copy.rotate} onClick={() => setRotation((value) => value + 90)}>
              <RotateCw />
            </ViewerButton>
            <ViewerButton label={copy.zoomOut} disabled={scale <= MIN_SCALE} onClick={() => zoomBy(-SCALE_STEP)}>
              <ZoomOut />
            </ViewerButton>
            <span className="w-14 shrink-0 text-center text-xs tabular-nums" aria-live="polite">
              {Math.round(scale * 100)}%
            </span>
            <ViewerButton label={copy.zoomIn} disabled={scale >= MAX_SCALE} onClick={() => zoomBy(SCALE_STEP)}>
              <ZoomIn />
            </ViewerButton>
            <ViewerButton label={copy.reset} onClick={resetTransform}>
              <Scan />
            </ViewerButton>
            {activeImage.downloadable !== false ? (
              <ViewerButton label={copy.download} onClick={handleDownload}>
                <Download />
              </ViewerButton>
            ) : null}
          </div>

          <button
            type="button"
            aria-label={copy.close}
            className="absolute top-4 right-4 z-20 flex size-10 items-center justify-center rounded-md bg-black/65 text-white transition-colors hover:bg-black/85 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
            onClick={() => onOpenChange(false)}
          >
            <X aria-hidden="true" className="size-5" />
          </button>

          {images.length > 1 ? (
            <>
              <ViewerNavigationButton label={copy.previous} side="left" onClick={() => moveTo(safeIndex - 1)}>
                <ChevronLeft />
              </ViewerNavigationButton>
              <ViewerNavigationButton label={copy.next} side="right" onClick={() => moveTo(safeIndex + 1)}>
                <ChevronRight />
              </ViewerNavigationButton>
            </>
          ) : null}

          <div
            className={scale > 1 ? "flex min-h-0 flex-1 touch-none cursor-grab items-center justify-center overflow-hidden active:cursor-grabbing" : "flex min-h-0 flex-1 items-center justify-center overflow-hidden"}
            onPointerDown={handlePointerDown}
            onPointerMove={handlePointerMove}
            onPointerUp={handlePointerUp}
            onPointerCancel={() => setDragStart(null)}
          >
            {imageFailed ? (
              <div className="flex flex-col items-center gap-3 text-white/70">
                <ImageOff aria-hidden="true" className="size-10" />
                <span className="text-sm">{copy.loadError}</span>
              </div>
            ) : (
              <img
                src={activeImage.src}
                alt={activeImage.alt}
                draggable={false}
                onError={handleImageError}
                className="max-h-[calc(100dvh-8rem)] max-w-[calc(100vw-2rem)] select-none object-contain transition-transform duration-150 sm:max-w-[calc(100vw-8rem)]"
                style={{
                  transform: `translate(${offset.x}px, ${offset.y}px) rotate(${rotation}deg) scale(${mirrored ? -scale : scale}, ${scale})`,
                }}
              />
            )}
          </div>

          <div className="pointer-events-none absolute right-0 bottom-5 left-0 flex justify-center px-20">
            <span className="max-w-full truncate rounded-sm bg-black/55 px-3 py-1.5 text-sm text-white/90">
              {images.length > 1 ? `${safeIndex + 1} / ${images.length} · ` : ""}
              {activeImage.alt}
            </span>
          </div>
        </CoreDialogContent>
      </CoreDialog>
    </>
  )
}

interface ViewerButtonProps {
  label: string
  disabled?: boolean
  onClick: () => void
  children: ReactNode
}

function ViewerButton({ label, disabled = false, onClick, children }: ViewerButtonProps) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      disabled={disabled}
      onClick={onClick}
      className="flex size-9 shrink-0 items-center justify-center rounded-sm text-white transition-colors hover:bg-white/15 focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-white disabled:cursor-not-allowed disabled:opacity-35 [&>svg]:size-4"
    >
      {children}
    </button>
  )
}

interface ViewerNavigationButtonProps {
  label: string
  side: "left" | "right"
  onClick: () => void
  children: ReactNode
}

function ViewerNavigationButton({ label, side, onClick, children }: ViewerNavigationButtonProps) {
  const className =
    side === "left"
      ? "absolute top-1/2 left-4 z-10 flex size-11 -translate-y-1/2 items-center justify-center rounded-md bg-black/55 text-white transition-colors hover:bg-black/80 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white [&>svg]:size-6"
      : "absolute top-1/2 right-4 z-10 flex size-11 -translate-y-1/2 items-center justify-center rounded-md bg-black/55 text-white transition-colors hover:bg-black/80 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white [&>svg]:size-6"
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      className={className}
    >
      {children}
    </button>
  )
}
