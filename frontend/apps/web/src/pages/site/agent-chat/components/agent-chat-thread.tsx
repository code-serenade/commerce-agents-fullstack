import { Bot, UserRound } from "lucide-react"
import { useEffect, useRef } from "react"
import { useTranslation } from "react-i18next"
import type { AgentChatMessage } from "../types"

export interface AgentChatThreadProps {
  messages: AgentChatMessage[]
}

export function AgentChatThread({ messages }: AgentChatThreadProps) {
  const { t } = useTranslation()
  const endRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth", block: "end" })
  }, [messages])

  return (
    <section className="min-h-0 flex-1 overflow-y-auto px-4 py-6 sm:px-6">
      <div className="mx-auto flex w-full max-w-3xl flex-col gap-5">
        {messages.map((message) => {
          const isUser = message.role === "user"

          return (
            <article
              key={message.id}
              className={`flex gap-3 ${isUser ? "justify-end" : "justify-start"}`}
            >
              {!isUser ? (
                <div className="flex size-9 shrink-0 items-center justify-center rounded-md bg-(--app-active-bg) text-(--app-active-text)">
                  <Bot className="size-5" />
                </div>
              ) : null}
              <div
                className={`min-w-0 max-w-[min(42rem,85%)] rounded-lg px-4 py-3 text-sm/6 ${
                  isUser
                    ? "bg-(--app-active-bg) text-(--app-active-text)"
                    : "border border-(--app-border) bg-(--app-surface) text-(--app-text)"
                }`}
              >
                {message.content ? (
                  <p className="whitespace-pre-wrap wrap-break-word">{message.content}</p>
                ) : (
                  <p className="text-(--app-muted-text)">
                    {t("agentChat.streaming")}
                  </p>
                )}
                {message.status === "streaming" ? (
                  <span className="mt-2 inline-block size-2 animate-pulse rounded-full bg-current" />
                ) : null}
                {message.status === "error" ? (
                  <p className="mt-2 text-xs font-medium text-destructive">
                    {t("agentChat.errorBadge")}
                  </p>
                ) : null}
              </div>
              {isUser ? (
                <div className="flex size-9 shrink-0 items-center justify-center rounded-md border border-(--app-border) bg-(--app-surface) text-(--app-muted-text)">
                  <UserRound className="size-5" />
                </div>
              ) : null}
            </article>
          )
        })}
        <div ref={endRef} />
      </div>
    </section>
  )
}
