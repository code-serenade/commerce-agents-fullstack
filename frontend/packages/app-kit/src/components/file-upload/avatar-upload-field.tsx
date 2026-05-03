import { useEffect, useRef, useState, type ReactNode } from "react"
import type { Area, Point } from "react-easy-crop"
import { ImageUp } from "lucide-react"
import { useTranslation } from "react-i18next"
import {
  getUploadAvatarSignApi,
  uploadWithSignedUrlApi,
  type UploadSignResponse,
} from "@workspace/services/api/file"
import { Button } from "@workspace/ui-components"
import { createCroppedImageFile } from "./image-crop"
import { ImageCropperModal } from "./image-cropper-modal"

export interface AvatarUploadResult {
  key: string
  uploadUrl: string
  file: File
  sign: UploadSignResponse
}

export interface AvatarUploadTriggerState {
  disabled: boolean
  uploading: boolean
  openFileDialog: () => void
}

export interface AvatarUploadFieldProps {
  value?: string
  alt: string
  fallback: ReactNode
  disabled?: boolean
  accept?: string
  outputSize?: number
  getSign?: (file: File) => Promise<UploadSignResponse>
  upload?: (file: File, sign: UploadSignResponse) => Promise<Response>
  onUploaded?: (result: AvatarUploadResult) => void | Promise<void>
  onError?: (error: Error) => void
  renderTrigger?: (state: AvatarUploadTriggerState) => ReactNode
}

export function AvatarUploadField({
  value,
  alt,
  fallback,
  disabled = false,
  accept = "image/*",
  outputSize = 512,
  getSign = () => getUploadAvatarSignApi(),
  upload = (file, sign) =>
    uploadWithSignedUrlApi(file, sign, {
      contentType: file.type || "image/png",
      contentDisposition: false,
    }),
  onUploaded,
  onError,
  renderTrigger,
}: AvatarUploadFieldProps) {
  const { t } = useTranslation("components")
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [uploading, setUploading] = useState(false)
  const [errorText, setErrorText] = useState("")
  const [cropOpen, setCropOpen] = useState(false)
  const [cropImageUrl, setCropImageUrl] = useState("")
  const [crop, setCrop] = useState<Point>({ x: 0, y: 0 })
  const [zoom, setZoom] = useState(1.2)
  const [croppedAreaPixels, setCroppedAreaPixels] = useState<Area | null>(null)

  useEffect(() => {
    return () => {
      if (cropImageUrl) {
        URL.revokeObjectURL(cropImageUrl)
      }
    }
  }, [cropImageUrl])

  const closeCropModal = () => {
    setCropOpen(false)
    setCrop({ x: 0, y: 0 })
    setZoom(1.2)
    setCroppedAreaPixels(null)

    if (cropImageUrl) {
      URL.revokeObjectURL(cropImageUrl)
      setCropImageUrl("")
    }
  }

  const openFileDialog = () => {
    if (disabled || uploading) {
      return
    }

    fileInputRef.current?.click()
  }

  const startCropAvatar = (file: File) => {
    if (!file.type.startsWith("image/")) {
      setErrorText(t("avatarUpload.errors.imageOnly"))
      return
    }

    const nextUrl = URL.createObjectURL(file)
    if (cropImageUrl) {
      URL.revokeObjectURL(cropImageUrl)
    }

    setErrorText("")
    setCropImageUrl(nextUrl)
    setCrop({ x: 0, y: 0 })
    setZoom(1.2)
    setCroppedAreaPixels(null)
    setCropOpen(true)
  }

  const handleConfirm = async () => {
    if (!cropImageUrl || !croppedAreaPixels) {
      return
    }

    setUploading(true)
    setErrorText("")
    try {
      const avatarFile = await createCroppedImageFile(
        cropImageUrl,
        croppedAreaPixels,
        {
          outputSize,
          mimeType: "image/png",
          fileName: `avatar-${Date.now()}.png`,
        }
      )
      const sign = await getSign(avatarFile)
      await upload(avatarFile, sign)
      await onUploaded?.({
        key: sign.key,
        uploadUrl: sign.upload_url,
        file: avatarFile,
        sign,
      })
      closeCropModal()
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err))
      setErrorText(error.message)
      onError?.(error)
    } finally {
      setUploading(false)
    }
  }

  const triggerState: AvatarUploadTriggerState = {
    disabled: disabled || uploading,
    uploading,
    openFileDialog,
  }

  return (
    <>
      <input
        ref={fileInputRef}
        type="file"
        accept={accept}
        className="hidden"
        onChange={(event) => {
          const file = event.currentTarget.files?.[0]
          if (file) {
            startCropAvatar(file)
          }
          event.currentTarget.value = ""
        }}
      />

      {renderTrigger ? (
        renderTrigger(triggerState)
      ) : (
        <div className="flex items-center gap-4">
          <span className="flex size-20 shrink-0 items-center justify-center overflow-hidden rounded-full bg-[#0f1724]">
            {value ? (
              <img
                src={value}
                alt={alt}
                className="block size-20 rounded-full object-cover"
              />
            ) : (
              <span className="flex size-20 items-center justify-center rounded-full bg-[#1c2431] text-xl font-semibold text-white">
                {fallback}
              </span>
            )}
          </span>
          <div className="min-w-0 space-y-2">
            <Button
              type="button"
              variant="outline"
              onClick={openFileDialog}
              disabled={triggerState.disabled}
            >
              <ImageUp className="size-4" />
              {uploading
                ? t("avatarUpload.uploading")
                : t("avatarUpload.select")}
            </Button>
            {errorText ? (
              <p className="text-sm text-destructive">{errorText}</p>
            ) : null}
          </div>
        </div>
      )}

      <ImageCropperModal
        open={cropOpen}
        imageUrl={cropImageUrl}
        crop={crop}
        zoom={zoom}
        confirming={uploading}
        onCropChange={setCrop}
        onZoomChange={setZoom}
        onCropComplete={setCroppedAreaPixels}
        onCancel={closeCropModal}
        onConfirm={() => void handleConfirm()}
        title={t("avatarUpload.crop.title")}
        description={t("avatarUpload.crop.description")}
        zoomLabel={t("avatarUpload.crop.zoom")}
        cancelLabel={t("avatarUpload.crop.cancel")}
        confirmLabel={t("avatarUpload.crop.confirm")}
        confirmingLabel={t("avatarUpload.crop.confirming")}
      />
    </>
  )
}
