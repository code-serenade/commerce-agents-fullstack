import { UserRound } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentRoleActivity } from "../view-model/agent-role-cluster-view-model"
import type { AgentChatMessage } from "../types"
import { shouldShowStudentNextActions } from "../view-model/student-next-actions-view-model"
import { AgentRolePortrait } from "./agent-role-portrait"
import { AssistantAnswer } from "./assistant-answer"
import { MessageContextSummary } from "./message-context-summary"
import {
  AgentErrorActions,
  AgentProgressCard,
  AgentStoppedCard,
  MessageActions,
  StudentNextActions,
  WelcomeCard,
} from "./thread-message-blocks"

export function AgentChatThreadMessage({
  errorRetryPrompt,
  message,
  onExamplePromptSelect,
  onNextActionPromptSubmit,
  previousUserContent,
}: {
  errorRetryPrompt?: string
  message: AgentChatMessage
  onExamplePromptSelect?: (prompt: string) => void
  onNextActionPromptSubmit?: (prompt: string) => void
  previousUserContent?: string
}) {
  const { t } = useTranslation()
  const isUser = message.role === "user"
  const isWelcome = message.id === "welcome" && !isUser
  const assistantAvatarActivity = createAssistantAvatarActivity(
    message.status,
    t("agentChat.thread.assistantAvatar.name"),
    t("agentChat.thread.assistantAvatar.summary")
  )
  const assistantAvatarLabel =
    message.status === "streaming"
      ? t("agentChat.thread.assistantAvatar.ariaLabelWorking")
      : t("agentChat.thread.assistantAvatar.ariaLabelReady")

  return (
    <article
      className={`flex gap-3 ${isUser ? "justify-end" : "justify-start"}`}
    >
      {!isUser ? (
        <div
          className="flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-(--agent-chat-strong-border) bg-(--agent-chat-elevated-strong) text-foreground shadow-[var(--agent-chat-avatar-shadow)]"
          aria-label={assistantAvatarLabel}
          title={t("agentChat.thread.assistantAvatar.title")}
        >
          <AgentRolePortrait
            activity={assistantAvatarActivity}
            animate={message.status === "streaming"}
            size="xs"
          />
        </div>
      ) : null}
      <div
        className={`flex w-full max-w-[min(45rem,88%)] min-w-0 flex-col ${
          isUser ? "items-end" : "items-start"
        }`}
      >
        <div
          className={`min-w-0 text-sm/6 ${
            isUser
              ? "rounded-2xl bg-(--agent-chat-user-message-background) px-4 py-3 text-(--agent-chat-user-message-foreground) shadow-[var(--agent-chat-button-shadow)]"
              : "text-foreground"
          }`}
        >
          {isWelcome ? (
            <WelcomeCard
              content={message.content}
              onSelectExamplePrompt={onExamplePromptSelect}
            />
          ) : message.content && isUser ? (
            <p className="wrap-break-word whitespace-pre-wrap">
              {message.content}
            </p>
          ) : message.content ? (
            <AssistantAnswer content={message.content} />
          ) : (
            <p className="text-muted-foreground">{t("agentChat.streaming")}</p>
          )}
          {message.status === "streaming" && !message.content ? (
            <AgentProgressCard content={message.content} />
          ) : null}
          {message.status === "stopped" ? <AgentStoppedCard /> : null}
          {message.status === "error" ? (
            <AgentErrorActions
              fallbackPrompt={errorRetryPrompt}
              onSelectPrompt={onExamplePromptSelect}
            />
          ) : null}
          {!isUser && message.content && !isWelcome ? (
            <MessageActions content={message.content} />
          ) : null}
          {shouldShowStudentNextActions(message) ? (
            <StudentNextActions
              message={message}
              onSelectPrompt={onNextActionPromptSubmit}
              previousUserContent={previousUserContent}
            />
          ) : null}
        </div>
        {isUser ? (
          <MessageContextSummary summary={message.contextSummary} />
        ) : null}
      </div>
      {isUser ? (
        <div className="flex size-9 shrink-0 items-center justify-center rounded-xl border border-(--agent-chat-strong-border) bg-(--agent-chat-elevated-strong) text-muted-foreground shadow-[var(--agent-chat-avatar-shadow)]">
          <UserRound className="size-5" />
        </div>
      ) : null}
    </article>
  )
}

function createAssistantAvatarActivity(
  status: AgentChatMessage["status"],
  name: string,
  summary: string
): AgentRoleActivity {
  const state = status === "streaming" ? "working" : "done"

  return {
    action: summary,
    agent: {
      name,
      nodeId: "box",
      state,
      status: state,
      summary,
    },
    contribution: summary,
    lane: 0,
    role: "box",
    state,
  }
}
