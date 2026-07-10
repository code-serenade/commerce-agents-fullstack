import { type MouseEvent, type ReactNode, useState } from "react"
import {
  Bot,
  Library,
  Megaphone,
  MessageCircleMore,
  MoreHorizontal,
  MessageSquareText,
  ShoppingBag,
  Sparkles,
  Trash2,
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
import { CommerceAgentLogo } from "@/components/brand"
import { webNavigationSections } from "@/navigation"
import { AgentChatProvider } from "@/pages/site/agent-chat/components/agent-chat-provider"
import { useAgentChatContext } from "@/pages/site/agent-chat/context/use-agent-chat-context"
import { useAuthDialog } from "@/components/auth/AuthDialogProvider"

export interface AgentChatShellProps {
  children: ReactNode
}

const CHAT_ROUTE_PATHS = ["/", "/commerce", "/agent"] as const

type SidebarNavigationMode = "chat" | "training"

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
  const isQuestionRoute = isAnyRouteActive(location.pathname, CHAT_ROUTE_PATHS)
  const [renameTarget, setRenameTarget] = useState<{
    id: string
    title: string
  } | null>(null)
  const [renameValue, setRenameValue] = useState("")
  const [navigationMode, setNavigationMode] =
    useState<SidebarNavigationMode>("chat")
  const user = useSelector((state: RootState) => state.auth.user)
  const { requestLogin } = useAuthDialog()
  const isLoggedIn = Boolean(user)
  const displayName = user?.name || t("web.shell.account.loginCta")
  const displayId = user
    ? user.display_id || user.email || t("web.shell.account.defaultId")
    : t("web.shell.account.guestId")
  const activeConversation = chat.conversations.find(
    (conversation) => conversation.id === chat.conversationId
  )
  const routeTitleKey = getRouteTitleKey(location.pathname)
  const routeDescriptionKey = getRouteDescriptionKey(location.pathname)
  const useInnerPageScroll = isCommercePageLayoutRoute(location.pathname)
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
    if (!user && ["/products", "/materials", "/campaigns"].includes(path)) { requestLogin({ pathname: path }); return }
    navigate(path)
  }

  const openNewConversation = () => {
    chat.startNewConversation()
    navigateTo("/commerce")
  }

  const openConversation = (conversationId: string) => {
    void chat.selectConversation(conversationId)
    navigateTo("/commerce")
  }

  const isRouteActive = (path: string) => {
    return isAgentChatRouteActive(location.pathname, path)
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

  const trailing = [<LanguageSwitch key="lang" />, <ThemeToggle key="theme" />]
  const newConversationItem = {
    key: "new-chat",
    icon: <MessageCircleMore />,
    label: t("agentChat.sidebar.newChat"),
    textSize: "lg" as const,
    active: !chat.conversationId && isRouteActive("/commerce"),
    onSelect: openNewConversation,
  }
  const functionNavigationItems = [
    {
      key: "capabilities",
      path: "/agent-capabilities",
      icon: <Bot />,
      label: t("agentChat.shell.navigation.capabilities"),
      textSize: "lg" as const,
      active: isRouteActive("/agent-capabilities"),
      onSelect: () => navigateTo("/agent-capabilities"),
    },
    {
      key: "question-ideas",
      path: "/question-ideas",
      icon: <Sparkles />,
      label: t("agentChat.shell.navigation.questionIdeas"),
      textSize: "lg" as const,
      active: isRouteActive("/question-ideas"),
      onSelect: () => navigateTo("/question-ideas"),
    },
    {
      key: "product-board",
      path: "/products",
      icon: <ShoppingBag />,
      label: t("agentChat.shell.navigation.productBoard"),
      textSize: "lg" as const,
      active: isRouteActive("/products"),
      onSelect: () => navigateTo("/products"),
    },
    {
      key: "materials",
      path: "/materials",
      icon: <Library />,
      label: t("agentChat.shell.navigation.materials"),
      textSize: "lg" as const,
      active: isRouteActive("/materials"),
      onSelect: () => navigateTo("/materials"),
    },
    {
      key: "campaigns",
      path: "/campaigns",
      icon: <Megaphone />,
      label: t("agentChat.shell.navigation.campaigns"),
      textSize: "lg" as const,
      active: isRouteActive("/campaigns"),
      onSelect: () => navigateTo("/campaigns"),
    },
  ]
  const visibleFunctionNavigationItems = isLoggedIn
    ? functionNavigationItems
    : functionNavigationItems.filter(
        (item) =>
          !["product-board", "materials", "campaigns"].includes(item.key)
      )
  const workspaceSection: SidebarShellSection = {
    key: "workspace",
    label: t("agentChat.shell.sections.workspace"),
    hideLabel: true,
    maxContentHeight: "lg",
    itemOverflow: {
      maxVisibleItems: 4,
      moreLabel: t("agentChat.sidebar.more"),
      lessLabel: t("agentChat.sidebar.less"),
    },
    items: [newConversationItem, ...visibleFunctionNavigationItems],
  }
  const recentsSection: SidebarShellSection = {
    key: "recents",
    label: t("agentChat.shell.sections.recents"),
    hideWhenSidebarCollapsed: true,
    maxContentHeight: "md",
    items: chat.conversations.length
      ? [
          ...chat.conversations.map((conversation) => ({
            key: conversation.id,
            iconTone: "muted" as const,
            icon: <MessageSquareText />,
            label: conversation.title,
            active: isQuestionRoute && conversation.id === chat.conversationId,
            onSelect: () => openConversation(conversation.id),
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
        ]
      : [
          {
            key: "empty-recents",
            iconTone: "muted" as const,
            icon: <MessageSquareText />,
            label: t("agentChat.sidebar.empty"),
          },
        ],
  }
  const trainingNavigationSections = webNavigationSections.map((section) => ({
    ...section,
    items: section.items.filter((item) => item.id !== "home"),
  }))
  const trainingSections: SidebarShellSection[] = trainingNavigationSections.map(
    (section) => ({
      key: section.id,
      label: t(section.labelKey),
      items: section.items.map((item) => ({
        key: item.id,
        icon: item.icon,
        label: t(item.labelKey),
        active: item.matcher
          ? item.matcher(location.pathname)
          : isRouteActive(item.path),
        onSelect: () => navigateTo(item.path),
      })),
    })
  )
  const sections: SidebarShellSection[] = !isLoggedIn
    ? [workspaceSection]
    : navigationMode === "chat"
      ? [workspaceSection, recentsSection]
      : trainingSections
  const navigationHeader = isLoggedIn ? (
    <SidebarNavigationModeTabs
      activeMode={navigationMode}
      chatLabel={t("agentChat.shell.tabs.chat")}
      trainingLabel={t("agentChat.shell.tabs.training")}
      onModeChange={setNavigationMode}
    />
  ) : undefined
  const mobileConversationItems = chat.conversations
    .slice(0, 5)
    .map((conversation) => ({
      key: conversation.id,
      label: conversation.title,
      href: "/commerce",
      active: isQuestionRoute && conversation.id === chat.conversationId,
      onSelect: () => openConversation(conversation.id),
    }))
  const mobileFunctionSubItems = visibleFunctionNavigationItems.map((item) => ({
    key: item.key,
    label: item.label,
    href: item.path,
    active: item.active,
    onSelect: item.onSelect,
  }))
  const mobileTrainingItems: MobileBottomNavItem[] = trainingNavigationSections.map(
    (section) => ({
      key: `training-${section.id}`,
      icon: section.items[0]?.icon,
      label: t(section.labelKey),
      active: section.items.some((item) =>
        item.matcher
          ? item.matcher(location.pathname)
          : isRouteActive(item.path)
      ),
      subItems: section.items.map((item) => ({
        key: item.id,
        label: t(item.labelKey),
        href: item.path,
        active: item.matcher
          ? item.matcher(location.pathname)
          : isRouteActive(item.path),
        onSelect: () => navigateTo(item.path),
      })),
    })
  )
  const mobileNavigationItems: MobileBottomNavItem[] = [
    {
      key: "workspace",
      icon: <Bot />,
      label: t("agentChat.shell.sections.workspace"),
      active:
        isRouteActive("/agent-capabilities") ||
        isRouteActive("/question-ideas") ||
        (!chat.conversationId && isRouteActive("/commerce")) ||
        isRouteActive("/products") ||
        isRouteActive("/materials") ||
        isRouteActive("/campaigns"),
      subItems: [
        {
          key: "new-chat",
          label: t("agentChat.sidebar.newChat"),
          href: "/commerce",
          active: !chat.conversationId && isRouteActive("/commerce"),
          onSelect: openNewConversation,
        },
        ...mobileFunctionSubItems,
      ],
    },
    ...(isLoggedIn
      ? [
          {
            key: "recents",
            icon: <MessageSquareText />,
            label: t("agentChat.shell.sections.recents"),
            active: isQuestionRoute && chat.conversationId !== null,
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
          ...mobileTrainingItems,
        ]
      : []),
  ]

  return (
    <>
      <SidebarShell
        defaultSidebarOpen
        contentScroll={
          isQuestionRoute || useInnerPageScroll ? "hidden" : "auto"
        }
        contentPadding="none"
        brandLogo={<CommerceAgentLogo />}
        brandTitle={t("agentChat.shell.brandTitle")}
        sections={sections}
        navigationHeader={navigationHeader}
        mobileNavigation={{
          label: t("web.shell.navigation.mobileLabel"),
          items: mobileNavigationItems,
        }}
        showSidebarRail={false}
        header={
          <TopBar
            title={
              isQuestionRoute
                ? (activeConversation?.title ??
                  t("agentChat.shell.topbarTitle"))
                : routeTitleKey
                  ? t(routeTitleKey)
                  : t("agentChat.shell.topbarTitle")
            }
            meta={
              routeDescriptionKey
                ? t(routeDescriptionKey)
                : t("agentChat.shell.topbarMeta")
            }
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
          onSelect: user ? undefined : () => requestLogin(),
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
            : undefined,
        }}
        onBrandSelect={() => navigate("/home")}
      >
        <div className="relative flex size-full min-h-0 flex-1 bg-(--agent-chat-page-background) text-foreground">
          <div
            aria-hidden="true"
            className="pointer-events-none absolute inset-0 bg-(image:--agent-chat-page-ambient)"
          />
          <div className="relative z-10 flex size-full min-h-0 flex-1">
            {children}
          </div>
        </div>
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

function isAnyRouteActive(pathname: string, paths: readonly string[]) {
  return paths.some((path) => isAgentChatRouteActive(pathname, path))
}

function isAgentChatRouteActive(pathname: string, path: string) {
  if (path === "/commerce") {
    return pathname === "/" || pathname === "/commerce" || pathname === "/agent"
  }

  return pathname === path
}

function getRouteTitleKey(pathname: string) {
  if (pathname === "/agent-capabilities") {
    return "agentChat.shell.navigation.capabilities"
  }
  if (pathname === "/question-ideas") {
    return "agentChat.shell.navigation.questionIdeas"
  }
  if (pathname === "/products") {
    return "agentChat.shell.navigation.productBoard"
  }
  if (pathname === "/materials") {
    return "agentChat.shell.navigation.materials"
  }
  if (pathname === "/campaigns") {
    return "agentChat.shell.navigation.campaigns"
  }
  if (pathname === "/platform-connect") {
    return "agentChat.shell.navigation.platform"
  }
  if (pathname === "/overview") {
    return "commerce.pages.overview.title"
  }
  if (pathname === "/operations") {
    return "commerce.pages.operations.title"
  }
  if (pathname === "/agent-output") {
    return "agentChat.shell.navigation.deliverables"
  }
  if (pathname === "/approvals") {
    return "agentChat.shell.navigation.approvals"
  }
  if (pathname === "/workspaces") {
    return "agentChat.shell.navigation.taskBoard"
  }
  if (pathname === "/profile") {
    return "pages:profile.title"
  }
  return null
}

function getRouteDescriptionKey(pathname: string) {
  if (pathname === "/agent-capabilities") {
    return "agentCapabilities.header.description"
  }
  if (pathname === "/question-ideas") {
    return "questionIdeas.header.description"
  }
  if (pathname === "/products") {
    return "commerce.pages.products.description"
  }
  if (pathname === "/materials") {
    return "commerce.pages.files.description"
  }
  if (pathname === "/campaigns") {
    return "commerce.pages.campaigns.description"
  }
  if (pathname === "/platform-connect") {
    return "commerce.pages.platform.description"
  }
  if (pathname === "/overview") {
    return "commerce.pages.overview.description"
  }
  if (pathname === "/operations") {
    return "commerce.pages.operations.description"
  }
  if (pathname === "/agent-output") {
    return "agentOutput.header.description"
  }
  if (pathname === "/approvals") {
    return "commerce.pages.approvals.description"
  }
  if (pathname === "/workspaces") {
    return "commerce.pages.workspaces.description"
  }
  return null
}

function isCommercePageLayoutRoute(pathname: string) {
  return [
    "/agent-capabilities",
    "/question-ideas",
    "/overview",
    "/materials",
    "/products",
    "/campaigns",
    "/workspaces",
    "/platform-connect",
    "/approvals",
    "/operations",
  ].some((path) => isAgentChatRouteActive(pathname, path))
}

function SidebarNavigationModeTabs({
  activeMode,
  chatLabel,
  trainingLabel,
  onModeChange,
}: {
  activeMode: SidebarNavigationMode
  chatLabel: string
  trainingLabel: string
  onModeChange: (mode: SidebarNavigationMode) => void
}) {
  const tabs: Array<{ key: SidebarNavigationMode; label: string }> = [
    { key: "chat", label: chatLabel },
    { key: "training", label: trainingLabel },
  ]

  return (
    <div className="grid grid-cols-2 gap-1 rounded-xl border border-sidebar-primary/25 bg-sidebar-primary/10 p-1 shadow-inner">
      {tabs.map((tab) => {
        const active = activeMode === tab.key

        return (
          <button
            key={tab.key}
            type="button"
            className={[
              "h-8 cursor-pointer rounded-lg px-2 text-xs font-semibold ring-sidebar-ring transition-colors focus-visible:ring-2 focus-visible:outline-none",
              active
                ? "bg-sidebar-primary text-sidebar-primary-foreground shadow-md ring-1 ring-sidebar-primary/45"
                : "text-sidebar-foreground/80 hover:bg-sidebar-accent hover:text-sidebar-foreground",
            ]
              .filter(Boolean)
              .join(" ")}
            aria-pressed={active}
            onClick={() => onModeChange(tab.key)}
          >
            {tab.label}
          </button>
        )
      })}
    </div>
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
      <DropdownMenuContent className="bg-white text-left dark:bg-popover">
        <DropdownMenuItem
          className="cursor-pointer justify-start text-left"
          onSelect={onRename}
        >
          {renameLabel}
        </DropdownMenuItem>
        <DropdownMenuItem
          className="cursor-pointer justify-start text-left"
          variant="destructive"
          onSelect={onDelete}
        >
          <Trash2 className="size-4" />
          {deleteLabel}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
