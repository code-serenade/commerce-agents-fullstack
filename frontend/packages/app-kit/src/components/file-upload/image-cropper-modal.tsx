import { createPortal } from "react-dom"
import Cropper from "react-easy-crop"
import type { Area, Point } from "react-easy-crop"
import "react-easy-crop/react-easy-crop.css"
import { Button, Slider } from "@workspace/ui-components"

export interface ImageCropperModalProps {
  open: boolean
  imageUrl: string
  crop: Point
  zoom: number
  minZoom?: number
  maxZoom?: number
  confirming?: boolean
  onCropChange: (value: Point) => void
  onZoomChange: (value: number) => void
  onCropComplete: (areaPixels: Area) => void
  onCancel: () => void
  onConfirm: () => void
  title: string
  description: string
  zoomLabel: string
  cancelLabel: string
  confirmLabel: string
  confirmingLabel: string
}

export function ImageCropperModal({
  open,
  imageUrl,
  crop,
  zoom,
  minZoom = 1,
  maxZoom = 4,
  confirming = false,
  onCropChange,
  onZoomChange,
  onCropComplete,
  onCancel,
  onConfirm,
  title,
  description,
  zoomLabel,
  cancelLabel,
  confirmLabel,
  confirmingLabel,
}: ImageCropperModalProps) {
  if (!open || typeof document === "undefined") {
    return null
  }

  return createPortal(
    <div
      data-app-image-cropper-modal="true"
      className="fixed inset-0 z-[2100] flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
    >
      <div className="w-full max-w-xl rounded-xl border border-[var(--border)] bg-[var(--surface)] p-5 text-[var(--surface-foreground)] shadow-[0_24px_80px_rgba(0,0,0,0.55)]">
        <h3 className="text-lg font-semibold text-foreground">{title}</h3>
        <p className="mt-2 text-sm text-muted-foreground">{description}</p>

        <div className="relative mt-5 h-80 overflow-hidden rounded-lg border border-[var(--border)] bg-black/20">
          <Cropper
            image={imageUrl}
            crop={crop}
            zoom={zoom}
            aspect={1}
            objectFit="cover"
            minZoom={minZoom}
            maxZoom={maxZoom}
            cropShape="round"
            showGrid={false}
            classes={{
              containerClassName: "cursor-grab active:cursor-grabbing",
            }}
            style={{
              cropAreaStyle: {
                border: "2px solid rgba(255,255,255,0.95)",
                color: "rgba(0, 0, 0, 0.72)",
              },
            }}
            onCropChange={onCropChange}
            onZoomChange={onZoomChange}
            onCropComplete={(_, areaPixels) => onCropComplete(areaPixels)}
          />
        </div>

        <div className="mt-5 space-y-2">
          <label className="text-sm font-medium text-foreground">
            {zoomLabel}
          </label>
          <Slider
            min={minZoom}
            max={maxZoom}
            step={0.01}
            value={[zoom]}
            onValueChange={(value) => onZoomChange(value[0] ?? zoom)}
            disabled={confirming}
          />
        </div>

        <div className="mt-6 flex justify-end gap-3">
          <Button
            type="button"
            variant="outline"
            onClick={onCancel}
            disabled={confirming}
          >
            {cancelLabel}
          </Button>
          <Button
            type="button"
            variant="primary"
            onClick={onConfirm}
            disabled={confirming}
          >
            {confirming ? confirmingLabel : confirmLabel}
          </Button>
        </div>
      </div>
    </div>,
    document.body
  )
}
