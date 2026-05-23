import {
  createProductAssetApi,
  getUploadDocumentSignApi,
  getUploadImageSignApi,
  uploadWithSignedUrlApi,
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
  const sign = isImageFile(file)
    ? await getUploadImageSignApi(file)
    : await getUploadDocumentSignApi(file)
  await uploadWithSignedUrlApi(file, sign, {
    contentType:
      file.type ||
      values.mimeType ||
      sign.headers["Content-Type"] ||
      "application/octet-stream",
  })

  return createProductAssetApi(productId, {
    assetType: values.assetType || "docs",
    bucket: "private",
    description: toOptionalString(values.description),
    fileName: values.fileName || file.name || sign.key.split("/").at(-1),
    fileSize: file.size ?? toNumberOrNull(values.fileSize),
    metadata: {},
    mimeType: file.type || values.mimeType || "application/octet-stream",
    objectKey: sign.key,
  })
}

function isImageFile(file: File) {
  return file.type.toLowerCase().startsWith("image/")
}
