import type { Area } from "react-easy-crop"

export async function createImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image()
    image.onload = () => resolve(image)
    image.onerror = reject
    image.src = src
  })
}

export async function createCroppedImageFile(
  imageSrc: string,
  cropPixels: Area,
  options?: {
    outputSize?: number
    mimeType?: string
    fileName?: string
  }
): Promise<File> {
  const outputSize = options?.outputSize ?? 512
  const mimeType = options?.mimeType ?? "image/png"
  const fileName = options?.fileName ?? `crop-${Date.now()}.png`

  const image = await createImage(imageSrc)
  const canvas = document.createElement("canvas")
  canvas.width = outputSize
  canvas.height = outputSize

  const ctx = canvas.getContext("2d")
  if (!ctx) {
    throw new Error("Canvas context unavailable")
  }

  ctx.drawImage(
    image,
    cropPixels.x,
    cropPixels.y,
    cropPixels.width,
    cropPixels.height,
    0,
    0,
    outputSize,
    outputSize
  )

  const blob = await new Promise<Blob>((resolve, reject) => {
    canvas.toBlob((nextBlob) => {
      if (!nextBlob) {
        reject(new Error("Failed to export cropped image"))
        return
      }
      resolve(nextBlob)
    }, mimeType)
  })

  return new File([blob], fileName, { type: mimeType })
}
