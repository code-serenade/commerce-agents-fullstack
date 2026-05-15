import { type MouseEvent, type ReactNode, useState } from "react"
import {
  Bot,
  ClipboardList,
  ClipboardCheck,
  Database,
  Edit3,
  FileText,
  GitBranch,
  LogIn,
  Megaphone,
  MoreHorizontal,
  MessageSquareText,
  PackageCheck,
  PlugZap,
  Search,
  ShieldAlert,
  ShoppingBag,
  Trash2,
  UserPlus,
  UserRound,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import { useDispatch, useSelector } from "react-redux"
import { useLocation } from "react-router"
import { useNavigate } from "react-router"
import {
  CopyableText,
  LanguageSwitch,
  type MobileBottomNavItem,
  SidebarShell,
  type SidebarShellSection,
  ThemeToggle,
  TopBar,
} from "@workspace/app-kit"
import {
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Input,
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
  const location = useLocation()
  const navigate = useNavigate()
  const { t } = useTranslation()
  const [renameTarget, setRenameTarget] = useState<{
    id: string
    title: string
  } | null>(null)
  const [renameValue, setRenameValue] = useState("")
  const [visibleConversationCount, setVisibleConversationCount] = useState(5)
  const [historyOpen, setHistoryOpen] = useState(true)
  const [functionNavOpen, setFunctionNavOpen] = useState(false)
  const [governanceOpen, setGovernanceOpen] = useState(false)
  const user = useSelector((state: RootState) => state.auth.user)
  const displayName = user?.name || t("web.shell.account.guestName")
  const displayId = user
    ? user.display_id || user.email || t("web.shell.account.defaultId")
    : t("web.shell.account.guestId")
  const activeConversation = chat.conversations.find(
    (conversation) => conversation.id === chat.conversationId
  )

  const handleLogout = () => {
    localStorage.removeItem("token")
    localStorage.removeItem("refreshToken")
    dispatch(resetAccess())
    dispatch(logout())
    navigate("/home", { replace: true })
  }

  const openRenameDialog = (conversation: { id: string; title: string }) => {
    setRenameTarget(conversation)
    setRenameValue(conversation.title)
  }

  const navigateTo = (path: string) => {
    navigate(path)
  }

  const isRouteActive = (path: string) => {
    if (path === "/commerce") {
      return (
        location.pathname === "/" ||
        location.pathname === "/commerce" ||
        location.pathname === "/agent"
      )
    }

    return location.pathname === path
  }

  const closeRenameDialog = () => {
    setRenameTarget(null)
    setRenameValue("")
  }

  const submitRename = () => {
    if (!renameTarget) return
    void chat.renameConversation(renameTarget.id, renameValue)
    closeRenameDialog()
  }

  const trailing = [
    <LanguageSwitch key="lang" />,
    <ThemeToggle key="theme" />,
  ]
  const recentConversations = chat.conversations.slice(
    0,
    visibleConversationCount
  )
  const hiddenConversationCount = Math.max(
    chat.conversations.length - recentConversations.length,
    0
  )
  const sections: SidebarShellSection[] = [
    {
      key: "create",
      label: t("agentChat.shell.sections.create"),
      items: [
        {
          key: "new-chat",
          icon: <Edit3 />,
          label: t("agentChat.shell.navigation.newChat"),
          active: !chat.conversationId,
          onSelect: chat.startNewConversation,
        },
      ],
    },
    {
      key: "recents",
      label: t("agentChat.shell.sections.recents"),
      collapsible: true,
      maxContentHeight: "md",
      open: historyOpen,
      toggleLabel: historyOpen
        ? t("agentChat.sidebar.collapseHistory")
        : t("agentChat.sidebar.expandHistory"),
      onOpenChange: setHistoryOpen,
      items: recentConversations.length
        ? [
            ...recentConversations.map((conversation) => ({
              key: conversation.id,
              icon: <MessageSquareText />,
              label: conversation.title,
              active: conversation.id === chat.conversationId,
              onSelect: () => chat.selectConversation(conversation.id),
              action: (
                <ConversationActionMenu
                  actionLabel={t("pages:agentChat.conversation.actionLabel")}
                  deleteLabel={t("common:actions.delete")}
                  disabled={chat.isStreaming}
                  onDelete={() => chat.deleteConversation(conversation.id)}
                  onRename={() => openRenameDialog(conversation)}
                  renameLabel={t("common:actions.rename")}
                />
              ),
            })),
            ...(hiddenConversationCount > 0
              ? [
                  {
                    key: "more-recents",
                    icon: <MoreHorizontal />,
                    label: t("agentChat.sidebar.moreHistory", {
                      count: Math.min(5, hiddenConversationCount),
                    }),
                    onSelect: () =>
                      setVisibleConversationCount((current) =>
                        Math.min(current + 5, chat.conversations.length)
                      ),
                  },
                ]
              : visibleConversationCount > 5 && chat.conversations.length > 5
                ? [
                    {
                      key: "less-recents",
                      icon: <MoreHorizontal />,
                      label: t("agentChat.sidebar.lessHistory"),
                      onSelect: () => setVisibleConversationCount(5),
                    },
                  ]
                : []),
          ]
        : [
            {
              key: "empty-recents",
              icon: <MessageSquareText />,
              label: t("agentChat.sidebar.empty"),
            },
          ],
    },
    {
      key: "workspace",
      label: t("agentChat.shell.sections.workspace"),
      collapsible: true,
      maxContentHeight: "lg",
      open: functionNavOpen,
      toggleLabel: functionNavOpen
        ? t("agentChat.sidebar.collapseFunctionNav")
        : t("agentChat.sidebar.expandFunctionNav"),
      onOpenChange: setFunctionNavOpen,
      items: [
        {
          key: "capabilities",
          icon: <Bot />,
          label: t("agentChat.shell.navigation.capabilities"),
          active: isRouteActive("/agent-capabilities"),
          onSelect: () => navigateTo("/agent-capabilities"),
        },
        {
          key: "smart-ops",
          icon: <GitBranch />,
          label: t("agentChat.shell.navigation.smartOps"),
          active: isRouteActive("/commerce"),
          onSelect: () => navigateTo("/commerce"),
        },
        {
          key: "product-board",
          icon: <ShoppingBag />,
          label: t("agentChat.shell.navigation.productBoard"),
          active: isRouteActive("/products"),
          onSelect: () => navigateTo("/products"),
        },
        {
          key: "campaigns",
          icon: <Megaphone />,
          label: t("agentChat.shell.navigation.campaigns"),
          active: isRouteActive("/campaigns"),
          onSelect: () => navigateTo("/campaigns"),
        },
        {
          key: "platform",
          icon: <PlugZap />,
          label: t("agentChat.shell.navigation.platform"),
          active: isRouteActive("/platform-connect"),
          onSelect: () => navigateTo("/platform-connect"),
        },
        {
          key: "market-research",
          icon: <Search />,
          label: t("agentChat.shell.navigation.marketResearch"),
          active: isRouteActive("/overview"),
          onSelect: () => navigateTo("/overview"),
        },
        {
          key: "briefing",
          icon: <FileText />,
          label: t("agentChat.shell.navigation.briefing"),
          active: isRouteActive("/operations"),
          onSelect: () => navigateTo("/operations"),
        },
        {
          key: "alerts",
          icon: <ShieldAlert />,
          label: t("agentChat.shell.navigation.alerts"),
          active: isRouteActive("/operations"),
          onSelect: () => navigateTo("/operations"),
        },
      ],
    },
    {
      key: "governance",
      label: t("agentChat.shell.sections.governance"),
      collapsible: true,
      defaultOpen: false,
      maxContentHeight: "sm",
      open: governanceOpen,
      toggleLabel: governanceOpen
        ? t("agentChat.sidebar.collapseGovernance")
        : t("agentChat.sidebar.expandGovernance"),
      onOpenChange: setGovernanceOpen,
      items: [
        {
          key: "deliverables",
          icon: <FileText />,
          label: t("agentChat.shell.navigation.deliverables"),
          active: isRouteActive("/agent-output"),
          onSelect: () => navigateTo("/agent-output"),
        },
        {
          key: "approvals",
          icon: <ClipboardCheck />,
          label: t("agentChat.shell.navigation.approvals"),
          active: isRouteActive("/approvals"),
          onSelect: () => navigateTo("/approvals"),
        },
        {
          key: "task-board",
          icon: <ClipboardList />,
          label: t("agentChat.shell.navigation.taskBoard"),
          active: isRouteActive("/workspaces"),
          onSelect: () => navigateTo("/workspaces"),
        },
        {
          key: "packages",
          icon: <PackageCheck />,
          label: t("agentChat.shell.navigation.packages"),
          active: isRouteActive("/packages"),
          onSelect: () => navigateTo("/packages"),
        },
        {
          key: "system-dashboard",
          icon: <Database />,
          label: t("agentChat.shell.navigation.systemDashboard"),
          active: isRouteActive("/overview"),
          onSelect: () => navigateTo("/overview"),
        },
      ],
    },
  ]
  const mobileConversationItems = chat.conversations.slice(0, 5).map(
    (conversation) => ({
      key: conversation.id,
      label: conversation.title,
      href: "/commerce",
      active: conversation.id === chat.conversationId,
      onSelect: () => {
        void chat.selectConversation(conversation.id)
        navigateTo("/commerce")
      },
    })
  )
  const mobileNavigationItems: MobileBottomNavItem[] = [
    {
      key: "new-chat",
      icon: <Edit3 />,
      label: t("agentChat.shell.navigation.newChat"),
      active: !chat.conversationId && isRouteActive("/commerce"),
      onSelect: () => {
        chat.startNewConversation()
        navigateTo("/commerce")
      },
    },
    {
      key: "recents",
      icon: <MessageSquareText />,
      label: t("agentChat.shell.sections.recents"),
      active: chat.conversationId !== null,
      subItems: mobileConversationItems.length
        ? mobileConversationItems
        : [
            {
              key: "empty-recents",
              label: t("agentChat.sidebar.empty"),
              href: "/commerce",
              onSelect: () => navigateTo("/commerce"),
            },
          ],
    },
    {
      key: "workspace",
      icon: <Bot />,
      label: t("agentChat.shell.sections.workspace"),
      active:
        isRouteActive("/agent-capabilities") ||
        isRouteActive("/commerce") ||
        isRouteActive("/products") ||
        isRouteActive("/campaigns") ||
        isRouteActive("/platform-connect"),
      subItems: [
        {
          key: "capabilities",
          label: t("agentChat.shell.navigation.capabilities"),
          href: "/agent-capabilities",
          active: isRouteActive("/agent-capabilities"),
          onSelect: () => navigateTo("/agent-capabilities"),
        },
        {
          key: "smart-ops",
          label: t("agentChat.shell.navigation.smartOps"),
          href: "/commerce",
          active: isRouteActive("/commerce"),
          onSelect: () => navigateTo("/commerce"),
        },
        {
          key: "product-board",
          label: t("agentChat.shell.navigation.productBoard"),
          href: "/products",
          active: isRouteActive("/products"),
          onSelect: () => navigateTo("/products"),
        },
        {
          key: "campaigns",
          label: t("agentChat.shell.navigation.campaigns"),
          href: "/campaigns",
          active: isRouteActive("/campaigns"),
          onSelect: () => navigateTo("/campaigns"),
        },
        {
          key: "platform",
          label: t("agentChat.shell.navigation.platform"),
          href: "/platform-connect",
          active: isRouteActive("/platform-connect"),
          onSelect: () => navigateTo("/platform-connect"),
        },
      ],
    },
    {
      key: "governance",
      icon: <ClipboardCheck />,
      label: t("agentChat.shell.sections.governance"),
      active:
        isRouteActive("/agent-output") ||
        isRouteActive("/approvals") ||
        isRouteActive("/workspaces") ||
        isRouteActive("/packages"),
      subItems: [
        {
          key: "deliverables",
          label: t("agentChat.shell.navigation.deliverables"),
          href: "/agent-output",
          active: isRouteActive("/agent-output"),
          onSelect: () => navigateTo("/agent-output"),
        },
        {
          key: "approvals",
          label: t("agentChat.shell.navigation.approvals"),
          href: "/approvals",
          active: isRouteActive("/approvals"),
          onSelect: () => navigateTo("/approvals"),
        },
        {
          key: "task-board",
          label: t("agentChat.shell.navigation.taskBoard"),
          href: "/workspaces",
          active: isRouteActive("/workspaces"),
          onSelect: () => navigateTo("/workspaces"),
        },
        {
          key: "packages",
          label: t("agentChat.shell.navigation.packages"),
          href: "/packages",
          active: isRouteActive("/packages"),
          onSelect: () => navigateTo("/packages"),
        },
      ],
    },
  ]

  return (
    <>
      <SidebarShell
        defaultSidebarOpen
        brandLogo={
          <div className="relative flex size-10 items-center justify-center rounded-xl bg-foreground font-mono text-base font-semibold text-background shadow-sm">
            CA
            <span className="absolute right-1 top-1 size-1.5 rounded-full bg-blue-500" />
          </div>
        }
        brandEyebrow="Commerce Agent"
        brandTitle={t("agentChat.shell.brandTitle")}
        brandDescription={t("agentChat.shell.brandDescription")}
        sections={sections}
        mobileNavigation={{
          label: t("web.shell.navigation.mobileLabel"),
          items: mobileNavigationItems,
        }}
        showSidebarRail={false}
        header={
          <TopBar
            title={activeConversation?.title ?? t("agentChat.shell.topbarTitle")}
            meta={t("agentChat.shell.topbarMeta")}
            showSidebarTrigger={false}
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
                  onSelect: () => navigate("/profile"),
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
      <Dialog open={Boolean(renameTarget)} onOpenChange={closeRenameDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {t("pages:agentChat.conversation.renameTitle")}
            </DialogTitle>
          </DialogHeader>
          <Input
            value={renameValue}
            placeholder={t("pages:agentChat.conversation.renamePlaceholder")}
            onValueChange={setRenameValue}
          />
          <DialogFooter>
            <Button variant="outline" onClick={closeRenameDialog}>
              {t("common:actions.cancel")}
            </Button>
            <Button onClick={submitRename} disabled={!renameValue.trim()}>
              {t("common:actions.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}

function ConversationActionMenu({
  actionLabel,
  deleteLabel,
  disabled,
  onDelete,
  onRename,
  renameLabel,
}: {
  actionLabel: string
  deleteLabel: string
  disabled: boolean
  onDelete: () => void
  onRename: () => void
  renameLabel: string
}) {
  const stopClick = (event: MouseEvent) => {
    event.preventDefault()
    event.stopPropagation()
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        disabled={disabled}
        label={actionLabel}
        size="compactIcon"
        variant="muted"
        onClick={stopClick}
      >
        <MoreHorizontal className="size-4" />
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuItem onSelect={onRename}>{renameLabel}</DropdownMenuItem>
        <DropdownMenuItem variant="destructive" onSelect={onDelete}>
          <Trash2 className="size-4" />
          {deleteLabel}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
