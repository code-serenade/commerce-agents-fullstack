import {
  createProductAssetApi,
  uploadDocumentToFileServiceApi,
  uploadImageToFileServiceApi,
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
  const upload = isImageFile(file)
    ? await uploadImageToFileServiceApi(file)
    : await uploadDocumentToFileServiceApi(file)

  return createProductAssetApi(productId, {
    assetType: values.assetType || "docs",
    bucket: "private",
    description: toOptionalString(values.description),
    fileName: values.fileName || file.name || upload.key.split("/").at(-1),
    fileSize: upload.size || file.size || toNumberOrNull(values.fileSize),
    metadata: {},
    mimeType: file.type || values.mimeType || "application/octet-stream",
    objectKey: upload.key,
  })
}

function isImageFile(file: File) {
  return file.type.toLowerCase().startsWith("image/")
}
