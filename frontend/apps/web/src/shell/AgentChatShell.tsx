import { type ReactNode } from "react"
import {
  ClipboardList,
  Code2,
  Database,
  Edit3,
  FileText,
  GitBranch,
  LogIn,
  Megaphone,
  MessageSquareText,
  PackageCheck,
  PlugZap,
  Search,
  ShieldAlert,
  ShoppingBag,
  UserPlus,
  UserRound,
  WalletCards,
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

  const trailing = [
    <LanguageSwitch key="lang" />,
    <ThemeToggle key="theme" />,
  ]
  const sections: SidebarShellSection[] = [
    {
      key: "create",
      label: "",
      items: [
        {
          key: "new-chat",
          icon: <Edit3 />,
          label: "新建会话",
          active: !chat.conversationId,
          onSelect: chat.startNewConversation,
        },
      ],
    },
    {
      key: "workspace",
      label: "运营工作台",
      items: [
        {
          key: "agent-swarm",
          icon: <GitBranch />,
          label: "多智能体调度",
          active: Boolean(chat.conversationId),
        },
        {
          key: "product-board",
          icon: <ShoppingBag />,
          label: "商品看板",
        },
        {
          key: "campaigns",
          icon: <Megaphone />,
          label: "营销活动",
        },
        {
          key: "deliverables",
          icon: <FileText />,
          label: "交付资料",
        },
        {
          key: "task-board",
          icon: <ClipboardList />,
          label: "执行清单",
        },
        {
          key: "market-research",
          icon: <Search />,
          label: "智能洞察",
        },
      ],
    },
    {
      key: "commerce-capabilities",
      label: "Commerce 能力",
      items: [
        {
          key: "douyin-ads",
          icon: <Megaphone />,
          label: "千川投放",
        },
        {
          key: "budget",
          icon: <WalletCards />,
          label: "预算核算",
        },
        {
          key: "ops-plan",
          icon: <ClipboardList />,
          label: "运营方案",
        },
        {
          key: "data-diagnosis",
          icon: <Database />,
          label: "数据诊断",
        },
        {
          key: "data-import",
          icon: <PlugZap />,
          label: "数据接入",
        },
        {
          key: "alerts",
          icon: <ShieldAlert />,
          label: "预警中心",
        },
        {
          key: "packages",
          icon: <PackageCheck />,
          label: "能力包管理",
        },
        {
          key: "more",
          icon: <Code2 />,
          label: t("agentChat.sidebar.more"),
        },
      ],
    },
    {
      key: "recents",
      label: "历史会话",
      items: [
        {
          key: "history",
          icon: <MessageSquareText />,
          label: "历史会话",
          active: chat.conversations.some(
            (conversation) => conversation.id === chat.conversationId
          ),
          subItems: chat.conversations.length
            ? chat.conversations.slice(0, 8).map((conversation) => ({
                key: conversation.id,
                href: "#",
                label: conversation.title,
                active: conversation.id === chat.conversationId,
                onSelect: () => chat.selectConversation(conversation.id),
              }))
            : [
                {
                  key: "empty-recents",
                  href: "#",
                  label: t("agentChat.sidebar.empty"),
                },
              ],
        },
      ],
    },
  ]

  return (
    <SidebarShell
      brandLogo={
        <div className="relative flex size-10 items-center justify-center rounded-xl bg-foreground font-mono text-base font-semibold text-background shadow-sm">
          CA
          <span className="absolute right-1 top-1 size-1.5 rounded-full bg-blue-500" />
        </div>
      }
      brandEyebrow="Commerce Agent"
      brandTitle="运营中枢"
      brandDescription="电商智能体工作台"
      sections={sections}
      header={
        <TopBar
          title={activeConversation?.title ?? "输入运营目标"}
          meta="Commerce 运营智能体 · 协作流程调度"
          leading={<GitBranch className="size-5" />}
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
