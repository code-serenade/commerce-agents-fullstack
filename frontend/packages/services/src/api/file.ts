import { request } from "./base"

export interface AccessSignQuery {
  key: string
}

export interface DeleteSignQuery {
  key: string
}

export interface UploadExtQuery {
  ext: string
}

export interface UploadSignQuery extends UploadExtQuery {
  filename: string
}

export interface SignedUploadHeaders {
  authorization: string
  "x-amz-date": string
  "x-amz-content-sha256": string
  "Content-Type"?: string
  "Content-Disposition"?: string
}

export interface UploadSignResponse {
  method: "PUT"
  upload_url: string
  key: string
  already_uploaded: boolean
  headers: SignedUploadHeaders
}

export interface ServerUploadResponse {
  key: string
  url: string
  sha256: string
  size: number
  already_uploaded: boolean
}

export interface DownloadSignResponse {
  method: "GET"
  download_url: string
  key: string
}

export interface DeleteSignResponse {
  method: "DELETE"
  delete_url: string
  key: string
  headers: SignedUploadHeaders
}

export const getAccessSignApi = async (
  query: AccessSignQuery
): Promise<DownloadSignResponse> => {
  return request<AccessSignQuery, DownloadSignResponse>({
    method: "GET",
    url: "/file/sign/access",
    body: query,
    group: "file",
  })
}

export const getDeleteSignApi = async (
  query: DeleteSignQuery
): Promise<DeleteSignResponse> => {
  return request<DeleteSignQuery, DeleteSignResponse>({
    method: "GET",
    url: "/file/sign/delete",
    body: query,
    group: "file",
  })
}

export const getUploadAvatarSignApi = async (): Promise<UploadSignResponse> => {
  return request<undefined, UploadSignResponse>({
    method: "GET",
    url: "/file/sign/upload/avatar",
    group: "file",
  })
}

export const getUploadDocumentSignApi = async (
  input: File | UploadSignQuery
): Promise<UploadSignResponse> => {
  const query = input instanceof File ? await createUploadSignQuery(input) : input

  return request<UploadSignQuery, UploadSignResponse>({
    method: "GET",
    url: "/file/sign/upload/document",
    body: query,
    group: "file",
  })
}

export const getUploadImageSignApi = async (
  input: File | UploadSignQuery
): Promise<UploadSignResponse> => {
  const query = input instanceof File ? await createUploadSignQuery(input) : input

  return request<UploadSignQuery, UploadSignResponse>({
    method: "GET",
    url: "/file/sign/upload/image",
    body: query,
    group: "file",
  })
}

export const uploadAvatarToFileServiceApi = async (
  file: File
): Promise<ServerUploadResponse> => {
  return uploadToFileServiceApi("/file/sign/upload/avatar", file)
}

export const uploadDocumentToFileServiceApi = async (
  file: File
): Promise<ServerUploadResponse> => {
  return uploadToFileServiceApi("/file/sign/upload/document", file)
}

export const uploadImageToFileServiceApi = async (
  file: File
): Promise<ServerUploadResponse> => {
  return uploadToFileServiceApi("/file/sign/upload/image", file)
}

export const uploadWithSignedUrlApi = async (
  file: File,
  sign: UploadSignResponse,
  options?: {
    contentType?: string
    contentDisposition?: string | false
    credentials?: RequestCredentials
  }
): Promise<Response> => {
  if (sign.already_uploaded) {
    return new Response(null, {
      status: 204,
      statusText: "Already Uploaded",
    })
  }

  const headers = new Headers({
    Authorization: sign.headers.authorization,
    "x-amz-date": sign.headers["x-amz-date"],
    "x-amz-content-sha256": sign.headers["x-amz-content-sha256"],
  })

  if (sign.headers["Content-Type"]) {
    headers.set("Content-Type", sign.headers["Content-Type"])
  } else if (options?.contentType) {
    headers.set("Content-Type", options.contentType)
  }
  if (options?.contentDisposition) {
    headers.set("Content-Disposition", options.contentDisposition)
  } else if (
    options?.contentDisposition !== false &&
    sign.headers["Content-Disposition"]
  ) {
    headers.set("Content-Disposition", sign.headers["Content-Disposition"])
  }

  const res = await fetch(sign.upload_url, {
    method: "PUT",
    headers,
    body: file,
    credentials: options?.credentials ?? "omit",
    mode: "cors",
  })

  if (!res.ok) {
    throw new Error(`Upload failed: ${res.status}`)
  }
  return res
}

export async function createUploadSignQuery(
  file: File
): Promise<UploadSignQuery> {
  return {
    filename: await calculateFileSha256(file),
    ext: getFileExtension(file),
  }
}

export async function calculateFileSha256(file: File): Promise<string> {
  const buffer = await file.arrayBuffer()
  const digest = await crypto.subtle.digest("SHA-256", buffer)

  return Array.from(new Uint8Array(digest))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("")
}

async function uploadToFileServiceApi(
  url: string,
  file: File
): Promise<ServerUploadResponse> {
  const query = new URLSearchParams({ ext: getFileExtension(file) })

  return request<Blob, ServerUploadResponse>({
    method: "PUT",
    url: `${url}?${query.toString()}`,
    body: file,
    group: "file",
  })
}

function getFileExtension(file: File): string {
  const extension = file.name.split(".").pop()?.trim().toLowerCase()

  if (!extension || extension === file.name.toLowerCase()) {
    throw new Error("File extension is required")
  }

  return extension
}

export const deleteWithSignedUrlApi = async (
  sign: DeleteSignResponse
): Promise<Response> => {
  const headers = new Headers({
    Authorization: sign.headers.authorization,
    "x-amz-date": sign.headers["x-amz-date"],
    "x-amz-content-sha256": sign.headers["x-amz-content-sha256"],
  })

  const res = await fetch(sign.delete_url, {
    method: "DELETE",
    headers,
  })

  if (!res.ok) {
    throw new Error(`Delete failed: ${res.status}`)
  }
  return res
}
