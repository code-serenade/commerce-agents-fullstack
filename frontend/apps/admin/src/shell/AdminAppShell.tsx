import { useEffect, useMemo, type ReactNode } from "react"
import { useTranslation } from "react-i18next"
import { useDispatch, useSelector } from "react-redux"
import { useLocation, useNavigate } from "react-router"
import { UserRound } from "lucide-react"
import {
  CopyableText,
  LanguageSwitch,
  type MobileBottomNavItem,
  NotificationDropdown,
  SidebarShell,
  ThemeToggle,
  WorkspaceTabs,
} from "@workspace/app-kit"
import { TopBar } from "@workspace/app-kit/top-bar"
import { logout } from "@/store/authSlice"
import { resetAccess } from "@/store/accessSlice"
import type { RootState } from "@/store"
import { useAdminNavigation } from "@/navigation"
import { getAdminNotifications } from "./admin-notifications"
import { useAdminNotificationSocket } from "./use-admin-notification-socket"
import { useWorkspaceTabs } from "./use-workspace-tabs"

export interface AdminAppShellProps {
  children: ReactNode
}

export function AdminAppShell({ children }: AdminAppShellProps) {
  const dispatch = useDispatch()
  const location = useLocation()
  const navigate = useNavigate()
  const { t } = useTranslation()
  const user = useSelector((state: RootState) => state.auth.user)
  const token = useSelector((state: RootState) => state.auth.token)
  const {
    currentItem,
    defaultPath,
    hasVisibleMenus,
    isLoading,
    labelsByPath,
    sections,
  } = useAdminNavigation()
  const notifications = getAdminNotifications(t)
  const mobileNavigationItems = useMemo<MobileBottomNavItem[]>(
    () =>
      sections.flatMap((section) =>
        section.items.map((item) => ({
          key: item.key,
          label: item.label,
          href: item.href,
          active: item.active,
          icon: item.icon,
          onSelect: item.onSelect,
          subItems: item.subItems?.map((subItem) => ({
            key: subItem.key,
            label: subItem.label,
            href: subItem.href,
            active: subItem.active,
            onSelect: subItem.onSelect,
          })),
        }))
      ),
    [sections]
  )
  const workspaceTabs = useWorkspaceTabs({
    currentPath: currentItem?.path ?? null,
    defaultPath,
    labelsByPath,
  })
  useAdminNotificationSocket({
    token: token ?? localStorage.getItem("token"),
  })

  const handleLogout = () => {
    localStorage.removeItem("token")
    localStorage.removeItem("refreshToken")
    dispatch(resetAccess())
    dispatch(logout())
    navigate("/login", { replace: true })
  }

  const isStandalonePage = location.pathname.startsWith("/profile")

  useEffect(() => {
    if (
      isLoading ||
      currentItem ||
      isStandalonePage ||
      !defaultPath ||
      !hasVisibleMenus
    ) {
      return
    }

    navigate(defaultPath, { replace: true })
  }, [
    currentItem,
    defaultPath,
    hasVisibleMenus,
    isLoading,
    isStandalonePage,
    navigate,
  ])

  const topBarTitle = isStandalonePage
    ? t("pages:profile.title")
    : currentItem?.label ?? t("admin.shell.brand.title")

  return (
    <SidebarShell
      brandEyebrow={t("admin.shell.brand.eyebrow")}
      brandTitle={t("admin.shell.brand.title")}
      brandDescription={t("admin.shell.brand.description")}
      sections={sections}
      mobileNavigation={{
        label: t("admin.shell.navigation.mobileLabel"),
        items: mobileNavigationItems,
      }}
      header={
        <>
          <TopBar
            title={topBarTitle}
            trailing={[
              <NotificationDropdown
                key="notifications"
                label={t("admin.shell.notifications.label")}
                panelTitle={t("admin.shell.notifications.title")}
                markAllLabel={t("admin.shell.notifications.actions.markAllRead")}
                emptyLabel={t("admin.shell.notifications.empty")}
                items={notifications}
              />,
              <LanguageSwitch key="lang" />,
              <ThemeToggle key="theme" />,
            ]}
          />
          <div className="hidden md:block">
            <WorkspaceTabs
              clearLabel={t("admin.shell.workspaceTabs.clear")}
              items={workspaceTabs.items}
              onClear={workspaceTabs.clear}
            />
          </div>
        </>
      }
      footerAccount={{
        avatarAlt: user?.name ?? t("admin.shell.account.fallbackName"),
        avatarSrc: user?.avatar,
        avatarFallback: (
          user?.name ?? t("admin.shell.account.fallbackInitial")
        )
          .charAt(0)
          .toUpperCase(),
        displayName: user?.name ?? t("admin.shell.account.fallbackName"),
        displayId: (
          <CopyableText
            value={
              user?.display_id ||
              user?.email ||
              t("admin.shell.account.fallbackId")
            }
            textClassName="block truncate"
          >
            {user?.display_id ||
              user?.email ||
              t("admin.shell.account.fallbackId")}
          </CopyableText>
        ),
        logout: {
          label: t("admin.shell.account.logout"),
          onSelect: handleLogout,
        },
        actions: [
          {
            icon: <UserRound />,
            label: t("admin.shell.account.actions.profile"),
            onSelect: () => navigate("/profile"),
          },
        ],
      }}
    >
      {children}
    </SidebarShell>
  )
}
