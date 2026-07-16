import { useState } from "react"
import { Image as ImageIcon } from "lucide-react"
import { useTranslation } from "react-i18next"
import { useQuery } from "@tanstack/react-query"
import { ImageViewer } from "@workspace/ui-components"
import { getConversationFileAccessApi } from "@/api"
import { fileDisplayName } from "../domain/logic"
import type { TrainingFileItem } from "../types"

const ACCESS_URL_STALE_TIME_MS = 5 * 60 * 1000

export function FilesImageThumbnail({ file }: { file: TrainingFileItem }) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const [index, setIndex] = useState(0)
  const accessQuery = useQuery({
    queryKey: ["commerce", "files", "access", file.id],
    queryFn: () => getConversationFileAccessApi(file.id),
    retry: 1,
    staleTime: ACCESS_URL_STALE_TIME_MS,
  })
  const downloadUrl = accessQuery.data?.downloadUrl

  if (!downloadUrl) {
    return (
      <div
        aria-hidden="true"
        className="flex size-12 shrink-0 items-center justify-center rounded-md border border-cyan-200 bg-cyan-50 text-cyan-600 dark:border-cyan-900 dark:bg-cyan-950/40 dark:text-cyan-300"
      >
        <ImageIcon className="size-5" strokeWidth={1.75} />
      </div>
    )
  }

  const fileName = fileDisplayName(file)

  return (
    <ImageViewer
      images={[
        {
          alt: fileName,
          downloadName: fileName,
          src: downloadUrl,
          thumbnailSrc: downloadUrl,
        },
      ]}
      index={index}
      labels={{
        close: t("commerce.files.imageViewer.close"),
        download: t("commerce.files.imageViewer.download"),
        loadError: t("commerce.files.imageViewer.loadError"),
        mirror: t("commerce.files.imageViewer.mirror"),
        next: t("commerce.files.imageViewer.next"),
        preview: t("commerce.files.imageViewer.preview"),
        previous: t("commerce.files.imageViewer.previous"),
        reset: t("commerce.files.imageViewer.reset"),
        rotate: t("commerce.files.imageViewer.rotate"),
        zoomIn: t("commerce.files.imageViewer.zoomIn"),
        zoomOut: t("commerce.files.imageViewer.zoomOut"),
      }}
      onIndexChange={setIndex}
      onOpenChange={setOpen}
      open={open}
      thumbnailSize="compact"
    />
  )
}
