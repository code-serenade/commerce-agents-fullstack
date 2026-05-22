import type { UpdateConversationFileExtractionRequest } from "@/api"

export const MAX_VISION_IMAGE_ATTACHMENTS = 4

const MAX_IMAGE_DATA_URL_CHARS = 8 * 1024 * 1024
const UNSUPPORTED_IMAGE_EXTRACTION_REASON =
  "Image upload is available for vision, but only PNG, JPEG, and WebP images can be sent to the vision model."
const IMAGE_TOO_LARGE_REASON =
  "Image is too large to send to the vision model. Upload a smaller PNG, JPEG, or WebP image."
const FAILED_IMAGE_EXTRACTION_REASON = "Browser image preparation failed."

export interface BrowserImageExtractionRequestResult {
  imageDataUrl?: string
  readable: boolean
  request: UpdateConversationFileExtractionRequest
}

export async function createBrowserImageExtractionRequest(
  file: File,
  metadata: Record<string, unknown>
): Promise<BrowserImageExtractionRequestResult> {
  if (!isVisionImageFile(file)) {
    return {
      readable: false,
      request: {
        contentKind: "unsupported",
        failureReason: UNSUPPORTED_IMAGE_EXTRACTION_REASON,
        metadata,
        parser: "browser_image",
        source: "browser_upload",
        status: "unsupported",
      },
    }
  }

  try {
    const imageDataUrl = await fileToDataUrl(file)
    if (imageDataUrl.length > MAX_IMAGE_DATA_URL_CHARS) {
      return {
        readable: false,
        request: {
          contentKind: "image_vision_input",
          failureReason: IMAGE_TOO_LARGE_REASON,
          metadata: {
            ...metadata,
            dataUrlChars: imageDataUrl.length,
            visionReady: false,
          },
          parser: "browser_image",
          source: "browser_upload",
          status: "failed",
        },
      }
    }

    return {
      imageDataUrl,
      readable: true,
      request: {
        contentKind: "image_vision_input",
        metadata: {
          ...metadata,
          dataUrlChars: imageDataUrl.length,
          imageDataUrl,
          visionDetail: "auto",
          visionReady: true,
        },
        parser: "browser_image",
        source: "browser_upload",
        status: "extracted",
        text: imageVisionPlaceholderText(file),
      },
    }
  } catch {
    return {
      readable: false,
      request: {
        contentKind: "image_vision_input",
        failureReason: FAILED_IMAGE_EXTRACTION_REASON,
        metadata,
        parser: "browser_image",
        source: "browser_upload",
        status: "failed",
      },
    }
  }
}

export function isImageFile(file: File) {
  return file.type.toLowerCase().startsWith("image/")
}

export function isVisionImageFile(file: File) {
  const type = file.type.toLowerCase()
  return (
    ["image/png", "image/jpeg", "image/webp"].includes(type) ||
    /\.(png|jpe?g|webp)$/i.test(file.name)
  )
}

function fileToDataUrl(file: File) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      if (typeof reader.result === "string") {
        resolve(reader.result)
      } else {
        reject(new Error("Image data URL result is not a string."))
      }
    }
    reader.onerror = () => reject(reader.error)
    reader.readAsDataURL(file)
  })
}

function imageVisionPlaceholderText(file: File) {
  return `Image attachment prepared for vision model: ${file.name || "image"}`
}
