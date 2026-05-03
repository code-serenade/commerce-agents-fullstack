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
  headers: SignedUploadHeaders
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
  query: UploadExtQuery
): Promise<UploadSignResponse> => {
  return request<UploadExtQuery, UploadSignResponse>({
    method: "GET",
    url: "/file/sign/upload/document",
    body: query,
    group: "file",
  })
}

export const getUploadImageSignApi = async (
  query: UploadExtQuery
): Promise<UploadSignResponse> => {
  return request<UploadExtQuery, UploadSignResponse>({
    method: "GET",
    url: "/file/sign/upload/image",
    body: query,
    group: "file",
  })
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
