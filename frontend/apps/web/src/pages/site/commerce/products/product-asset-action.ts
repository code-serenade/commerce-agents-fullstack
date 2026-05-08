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
    return createProductAssetApi(productId, {
      assetType: values.assetType || "docs",
      bucket: "private",
      description: toOptionalString(values.description),
      fileName: values.fileName,
      fileSize: toNumberOrNull(values.fileSize),
      metadata: {},
      mimeType: toOptionalString(values.mimeType),
      objectKey: values.objectKey,
    })
  }

  const extension = file.name.split(".").pop() || "bin"
  const sign = await presignProductAssetUploadApi(productId, {
    contentType: file.type || "application/octet-stream",
    extension,
  })
  await uploadProductAssetFileApi(file, sign)

  return createProductAssetApi(productId, {
    assetType: values.assetType || "docs",
    bucket: sign.bucket,
    description: toOptionalString(values.description),
    fileName: values.fileName || file.name,
    fileSize: file.size,
    metadata: {},
    mimeType: file.type || values.mimeType || "application/octet-stream",
    objectKey: sign.objectKey,
  })
}
