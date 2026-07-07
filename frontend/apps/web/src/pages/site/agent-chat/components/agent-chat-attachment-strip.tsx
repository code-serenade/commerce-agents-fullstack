import {
  CheckCircle2,
  Eye,
  Image,
  Loader2,
  Paperclip,
  XCircle,
} from "lucide-react"
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
              : attachment.status === "unsupported"
                ? XCircle
                : Loader2
        const KindIcon = attachment.kind === "image" ? Image : Paperclip
        return (
          <span
            key={attachment.id}
            className="flex max-w-56 min-w-0 items-center gap-1.5 rounded-xl border border-(--agent-chat-soft-border) bg-(--agent-chat-accent-soft) px-2.5 py-1 text-xs text-(--agent-chat-accent-foreground)"
            title={attachment.unsupportedReason || attachment.name}
          >
            <Icon
              className={`size-3.5 shrink-0 ${
                attachment.status === "uploading" ||
                attachment.status === "pending"
                  ? "animate-spin"
                  : ""
              }`}
            />
            <KindIcon className="size-3.5 shrink-0" />
            <span className="truncate">{attachment.name}</span>
            {attachment.imageDataUrl ? (
              <Eye className="size-3.5 shrink-0" />
            ) : null}
            <span className="shrink-0">
              {t(`agentChat.composer.attachments.${attachment.status}`)}
            </span>
          </span>
        )
      })}
    </div>
  )
}
