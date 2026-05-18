import { CheckCircle2, Loader2, Paperclip, XCircle } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentChatAttachmentItem } from "../types"

export function AgentChatAttachmentStrip({
  attachments,
}: {
  attachments: AgentChatAttachmentItem[]
}) {
  const { t } = useTranslation()

  if (!attachments.length) return null

  return (
    <div
      className="flex min-w-0 flex-wrap gap-1.5 px-1"
      data-composer-interactive="true"
    >
      {attachments.slice(-5).map((attachment) => {
        const Icon =
          attachment.status === "uploaded"
            ? CheckCircle2
            : attachment.status === "failed"
              ? XCircle
              : Loader2
        return (
          <span
            key={attachment.id}
            className="flex max-w-56 min-w-0 items-center gap-1.5 rounded-full border border-border bg-muted px-2.5 py-1 text-xs text-muted-foreground"
            title={attachment.name}
          >
            <Icon
              className={`size-3.5 shrink-0 ${
                attachment.status === "uploading" ? "animate-spin" : ""
              }`}
            />
            <Paperclip className="size-3.5 shrink-0" />
            <span className="truncate">{attachment.name}</span>
            <span className="shrink-0">
              {t(`agentChat.composer.attachments.${attachment.status}`)}
            </span>
          </span>
        )
      })}
    </div>
  )
}
