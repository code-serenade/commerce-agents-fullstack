import { type MouseEvent, type ReactNode, useEffect, useState } from "react"
import {
  Bot,
  GraduationCap,
  Library,
  Megaphone,
  MessageCircleMore,
  MoreHorizontal,
  MessageSquareText,
  Pencil,
  School,
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
const PROTECTED_MENU_ROUTE_PATHS = [
  "/overview",
  "/products",
  "/materials",
  "/campaigns",
  "/workspaces",
  "/platform-connect",
  "/approvals",
  "/operations",
  "/agent-output",
  "/learning-progress",
  "/profile",
  "/team-school",
] as const
const HIDDEN_AGENT_CHAT_NAVIGATION_ITEM_IDS = new Set([
  "workspaces",
  "platform-connect",
  "approvals",
  "operations",
])

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
  const user = useSelector((state: RootState) => state.auth.user)
  const { requestLogin } = useAuthDialog()
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
  const useFixedContentViewport = isQuestionRoute || useInnerPageScroll
  const useDocumentPageSurface = isDocumentPageSurfaceRoute(location.pathname)
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
    if (!user && isProtectedMenuRoute(path)) {
      requestLogin({ pathname: path })
      return
    }
    navigate(path)
  }

  const openNewConversation = () => {
    chat.startNewConversation()
    navigateTo("/commerce")
  }

  useEffect(() => {
    const handleNewConversationShortcut = (event: KeyboardEvent) => {
      if (event.defaultPrevented) return
      if (event.key.toLowerCase() !== "k") return
      if ((!event.ctrlKey && !event.metaKey) || event.altKey || event.shiftKey)
        return

      event.preventDefault()
      chat.startNewConversation()
      navigate("/commerce")
    }

    window.addEventListener("keydown", handleNewConversationShortcut)
    return () => {
      window.removeEventListener("keydown", handleNewConversationShortcut)
    }
  }, [chat, navigate])

  const openConversation = (conversationId: string) => {
    navigateTo(`/commerce/${conversationId}`)
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

  const trailing = [
    <Button
      key="learning"
      size="icon"
      variant="ghost"
      aria-label={t("web.shell.navigation.learningProgress")}
      title={t("web.shell.navigation.learningProgress")}
      onClick={() =>
        user
          ? navigateTo("/learning-progress")
          : requestLogin({ pathname: "/learning-progress" })
      }
    >
      <GraduationCap className="size-4" />
    </Button>,
    <LanguageSwitch key="lang" />,
    <ThemeToggle key="theme" />,
  ]
  const newConversationItem = {
    key: "new-chat",
    icon: <MessageCircleMore />,
    label: (
      <span className="flex min-w-0 flex-1 items-center gap-2">
        <span className="min-w-0 flex-1 truncate">
          {t("agentChat.sidebar.newChat")}
        </span>
        <span
          aria-hidden="true"
          className="shrink-0 text-xs font-normal text-sidebar-foreground/45 group-data-[collapsible=icon]:hidden"
        >
          Ctrl K
        </span>
      </span>
    ),
    textSize: "lg" as const,
    variant: "prominentAction" as const,
    active: !chat.conversationId && isRouteActive("/commerce"),
    onSelect: openNewConversation,
  }
  const functionNavigationItems = [
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
  ]
  const functionNavigationPaths = new Set(
    functionNavigationItems.map((item) => item.path)
  )
  const trainingNavigationSections = webNavigationSections.map((section) => ({
    ...section,
    items: section.items.filter(
      (item) =>
        item.id !== "home" &&
        item.id !== "overview" &&
        !HIDDEN_AGENT_CHAT_NAVIGATION_ITEM_IDS.has(item.id)
    ),
  }))
  const trainingNavigationItems = trainingNavigationSections.flatMap(
    (section) =>
      section.items
        .filter((item) => !functionNavigationPaths.has(item.path))
        .map((item) => ({
          key: item.id,
          path: item.path,
          icon: item.icon,
          label: t(item.labelKey),
          active: item.matcher
            ? item.matcher(location.pathname)
            : isRouteActive(item.path),
          onSelect: () => navigateTo(item.path),
        }))
  )
  const workspaceSection: SidebarShellSection = {
    key: "workspace",
    label: t("agentChat.shell.sections.workspace"),
    hideLabel: true,
    leadingItems: [newConversationItem],
    leadingItemsFixed: true,
    itemOverflow: {
      maxVisibleItems: 4,
      moreLabel: t("agentChat.sidebar.more"),
      lessLabel: t("agentChat.sidebar.less"),
    },
    items: [...functionNavigationItems, ...trainingNavigationItems],
  }
  const recentsSection: SidebarShellSection = {
    key: "recents",
    label: t("agentChat.shell.sections.recents"),
    collapsible: true,
    contentClassName: "agent-chat-history-scrollarea",
    defaultOpen: true,
    fillRemaining: true,
    hideWhenSidebarCollapsed: true,
    toggleLabel: t("agentChat.shell.sections.recents"),
    items: chat.conversations.length
      ? [
          ...chat.conversations.map((conversation) => ({
            key: conversation.id,
            iconSize: "xs" as const,
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
            label: user
              ? t("agentChat.sidebar.empty")
              : t("agentChat.sidebar.guestEmpty"),
            variant: "plainText" as const,
          },
        ],
  }
  const sections: SidebarShellSection[] = [workspaceSection, recentsSection]
  const mobileConversationItems = chat.conversations.map((conversation) => ({
    key: conversation.id,
    label: conversation.title,
    href: `/commerce/${conversation.id}`,
    active: isQuestionRoute && conversation.id === chat.conversationId,
    onSelect: () => openConversation(conversation.id),
  }))
  const mobileFunctionSubItems = [
    ...functionNavigationItems,
    ...trainingNavigationItems,
  ].map((item) => ({
    key: item.key,
    label: item.label,
    href: item.path,
    active: item.active,
    onSelect: item.onSelect,
  }))
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
              label: user
                ? t("agentChat.sidebar.empty")
                : t("agentChat.sidebar.guestEmpty"),
              href: "/commerce",
              onSelect: () => navigateTo("/commerce"),
            },
          ],
    },
  ]

  return (
    <>
      <SidebarShell
        defaultSidebarOpen
        contentScroll={useFixedContentViewport ? "hidden" : "auto"}
        contentPadding="none"
        brandLogo={
          <CommerceAgentLogo className="size-7 rounded-md" size="sm" />
        }
        brandTitle={t("agentChat.shell.brandTitle")}
        sections={sections}
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
                {
                  icon: <School />,
                  label: t("web.shell.account.actions.teamSchool"),
                  onSelect: () => navigate("/team-school"),
                },
              ]
            : undefined,
        }}
        onBrandSelect={() => navigate("/home")}
      >
        <div
          className={[
            "relative text-foreground",
            useDocumentPageSurface
              ? "bg-(--document-page-background)"
              : "bg-(--agent-chat-page-background)",
            useFixedContentViewport
              ? "flex size-full min-h-0 flex-1"
              : "min-h-full w-full",
          ].join(" ")}
        >
          <div
            aria-hidden="true"
            className={[
              "pointer-events-none absolute inset-0",
              useDocumentPageSurface
                ? "bg-(image:--document-page-ambient)"
                : "bg-(image:--agent-chat-page-ambient) bg-[length:100%_661px] bg-top bg-no-repeat",
            ].join(" ")}
          />
          <div
            className={[
              "relative z-10",
              useFixedContentViewport
                ? "flex size-full min-h-0 flex-1"
                : "min-h-full w-full",
            ].join(" ")}
          >
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

function isProtectedMenuRoute(pathname: string) {
  return PROTECTED_MENU_ROUTE_PATHS.some((path) =>
    isAgentChatRouteActive(pathname, path)
  )
}

function isAgentChatRouteActive(pathname: string, path: string) {
  if (path === "/commerce") {
    return (
      pathname === "/" ||
      pathname === "/commerce" ||
      pathname.startsWith("/commerce/") ||
      pathname === "/agent"
    )
  }
  if (path === "/workspaces") {
    return isWorkspacesRoute(pathname)
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
  if (pathname === "/learning-progress") {
    return "learningProgress.title"
  }
  if (pathname === "/approvals") {
    return "agentChat.shell.navigation.approvals"
  }
  if (isWorkspacesRoute(pathname)) {
    return "agentChat.shell.navigation.taskBoard"
  }
  if (pathname === "/profile") {
    return "pages:profile.title"
  }
  if (pathname === "/team-school") {
    return "profile.teamSchoolPage.title"
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
  if (pathname === "/learning-progress") {
    return "learningProgress.description"
  }
  if (pathname === "/approvals") {
    return "commerce.pages.approvals.description"
  }
  if (isWorkspacesRoute(pathname)) {
    return "commerce.pages.workspaces.description"
  }
  if (pathname === "/team-school") {
    return "profile.teamSchoolPage.description"
  }
  return null
}

function isCommercePageLayoutRoute(pathname: string) {
  if (isWorkspacesRoute(pathname)) {
    return true
  }

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
    "/agent-output",
    "/learning-progress",
  ].some((path) => isAgentChatRouteActive(pathname, path))
}

function isDocumentPageSurfaceRoute(pathname: string) {
  if (isWorkspacesRoute(pathname)) {
    return true
  }

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
    "/agent-output",
  ].some((path) => isAgentChatRouteActive(pathname, path))
}

function isWorkspacesRoute(pathname: string) {
  return pathname === "/workspaces" || pathname.startsWith("/workspaces/")
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
        className="bg-transparent! hover:bg-transparent! hover:text-sidebar-foreground! focus:bg-transparent! active:bg-transparent! data-[state=open]:bg-transparent! data-[state=open]:text-sidebar-foreground!"
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
          <Pencil className="size-4" />
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
