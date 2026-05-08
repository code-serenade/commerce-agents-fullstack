import {
  createProductAssetApi,
  presignProductAssetUploadApi,
  uploadProductAssetFileApi,
} from "@/api"
import {
  toNumberOrNull,
  toOptionalString,
  type SimpleFormValues,
} from "../components/commerce-ui"

export async function createAssetRecord({
  file,
  productId,
  values,
}: {
  file: File | null
  productId: string
  values: SimpleFormValues
}) {
  if (!file) {
    throw new Error("Asset file is required")
  }
  const extension =
    extensionFromFileName(file.name) ||
    extensionFromFileName(values.fileName) ||
    extensionFromMimeType(values.mimeType) ||
    "bin"
  const sign = await presignProductAssetUploadApi(productId, {
    contentType: file.type || values.mimeType || "application/octet-stream",
    extension,
  })
  await uploadProductAssetFileApi(file, sign)

  return createProductAssetApi(productId, {
    assetType: values.assetType || "docs",
    bucket: sign.bucket,
    description: toOptionalString(values.description),
    fileName: values.fileName || file.name || sign.objectKey.split("/").at(-1),
    fileSize: file.size ?? toNumberOrNull(values.fileSize),
    metadata: {},
    mimeType: file.type || values.mimeType || "application/octet-stream",
    objectKey: sign.objectKey,
  })
}

function extensionFromFileName(fileName: string | undefined) {
  const match = fileName?.match(/\.([^.]+)$/)
  return match?.[1] || null
}

function extensionFromMimeType(mimeType: string | undefined) {
  const subtype = mimeType?.split("/")[1]?.split(";")[0]?.trim()
  return subtype || null
}
