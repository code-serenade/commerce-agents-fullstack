import { type ReactNode } from "react"
import {
  Edit2,
  Edit3,
  Trash2,
  LogIn,
  MessageSquareText,
  MoreHorizontal,
  Search,
  UserPlus,
  UserRound,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import { useDispatch, useSelector } from "react-redux"
import { useNavigate } from "react-router"
import {
  CopyableText,
  LanguageSwitch,
  SidebarShell,
  type SidebarShellSection,
  ThemeToggle,
  TopBar,
} from "@workspace/app-kit"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@workspace/ui-components"
import type { RootState } from "@/store"
import { logout } from "@/store/authSlice"
import { resetAccess } from "@/store/accessSlice"
import { AgentChatProvider } from "@/pages/site/agent-chat/components/agent-chat-provider"
import { useAgentChatContext } from "@/pages/site/agent-chat/context/use-agent-chat-context"

export interface AgentChatShellProps {
  children: ReactNode
}

export function AgentChatShell({ children }: AgentChatShellProps) {
  return (
    <AgentChatProvider>
      <AgentChatShellContent>{children}</AgentChatShellContent>
    </AgentChatProvider>
  )
}

function AgentChatShellContent({ children }: AgentChatShellProps) {
  const chat = useAgentChatContext()
  const dispatch = useDispatch()
  const navigate = useNavigate()
  const { t } = useTranslation()
  const user = useSelector((state: RootState) => state.auth.user)
  const displayName = user?.name || t("web.shell.account.guestName")
  const displayId = user
    ? user.display_id || user.email || t("web.shell.account.defaultId")
    : t("web.shell.account.guestId")

  const handleLogout = () => {
    localStorage.removeItem("token")
    localStorage.removeItem("refreshToken")
    dispatch(resetAccess())
    dispatch(logout())
    navigate("/home", { replace: true })
  }

  const handleRenameConversation = (conversation: {
    id: string
    title: string
  }) => {
    const nextTitle = window.prompt(
      t("agentChat.sidebar.actions.renamePrompt"),
      conversation.title
    )
    if (nextTitle === null) return
    void chat.renameConversation(conversation.id, nextTitle)
  }

  const handleDeleteConversation = (conversation: {
    id: string
    title: string
  }) => {
    const confirmed = window.confirm(
      t("agentChat.sidebar.actions.deleteConfirm", {
        title: conversation.title,
      })
    )
    if (!confirmed) return
    void chat.deleteConversation(conversation.id)
  }

  const trailing = [
    <LanguageSwitch key="lang" />,
    <ThemeToggle key="theme" />,
  ]
  const sections: SidebarShellSection[] = [
    {
      key: "chat",
      label: t("agentChat.sidebar.chat"),
      items: [
        {
          key: "new-chat",
          icon: <Edit3 />,
          label: t("agentChat.sidebar.newChat"),
          active: !chat.conversationId,
          onSelect: chat.startNewConversation,
        },
        {
          key: "search-chats",
          icon: <Search />,
          label: t("agentChat.sidebar.search"),
        },
        {
          key: "more",
          icon: <MoreHorizontal />,
          label: t("agentChat.sidebar.more"),
        },
      ],
    },
    {
      key: "recents",
      label: t("agentChat.sidebar.recents"),
      items: chat.conversations.length
        ? chat.conversations.map((conversation) => ({
            key: conversation.id,
            icon: <MessageSquareText />,
            label: conversation.title,
            action: (
              <ConversationActionsMenu
                disabled={chat.isStreaming}
                onDelete={() => handleDeleteConversation(conversation)}
                onRename={() => handleRenameConversation(conversation)}
              />
            ),
            actionLabel: t("agentChat.sidebar.actions.open"),
            active: conversation.id === chat.conversationId,
            onSelect: () => chat.selectConversation(conversation.id),
          }))
        : [
            {
              key: "empty-recents",
              icon: <MessageSquareText />,
              label: t("agentChat.sidebar.empty"),
            },
          ],
    },
  ]

  return (
    <SidebarShell
      brandEyebrow={t("web.shell.brand.eyebrow")}
      brandTitle={t("web.shell.brand.title")}
      brandDescription={t("web.shell.brand.description")}
      sections={sections}
      header={
        <TopBar
          title={t("web.shell.navigation.agent")}
          meta={t("agentChat.sidebar.chat")}
          trailing={trailing}
        />
      }
      footerAccount={{
        avatarAlt: displayName,
        avatarSrc: user?.avatar,
        avatarFallback: user
          ? displayName.charAt(0).toUpperCase()
          : t("web.shell.account.guestFallback"),
        displayName,
        displayId: user ? (
          <CopyableText value={displayId} textClassName="block truncate">
            {displayId}
          </CopyableText>
        ) : (
          <span className="block truncate">{displayId}</span>
        ),
        logout: user
          ? {
              label: t("web.shell.account.logout"),
              onSelect: handleLogout,
            }
          : undefined,
        actions: user
          ? [
              {
                icon: <UserRound />,
                label: t("web.shell.account.actions.profile"),
                onSelect: () => chat.startNewConversation(),
              },
            ]
          : [
              {
                icon: <LogIn />,
                label: t("web.shell.account.login"),
                onSelect: () => navigate("/login"),
              },
              {
                icon: <UserPlus />,
                label: t("web.shell.account.register"),
                onSelect: () => navigate("/register"),
              },
            ],
      }}
    >
      {children}
    </SidebarShell>
  )
}

interface ConversationActionsMenuProps {
  disabled: boolean
  onDelete: () => void
  onRename: () => void
}

function ConversationActionsMenu({
  disabled,
  onDelete,
  onRename,
}: ConversationActionsMenuProps) {
  const { t } = useTranslation()

  return (
    <DropdownMenu>
      <DropdownMenuTrigger>
        <MoreHorizontal className="size-4" aria-hidden="true" />
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuItem
          disabled={disabled}
          onSelect={() => {
            onRename()
          }}
        >
          <Edit2 className="size-4" />
          {t("agentChat.sidebar.actions.rename")}
        </DropdownMenuItem>
        <DropdownMenuItem
          disabled={disabled}
          variant="destructive"
          onSelect={() => {
            onDelete()
          }}
        >
          <Trash2 className="size-4" />
          {t("agentChat.sidebar.actions.delete")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
