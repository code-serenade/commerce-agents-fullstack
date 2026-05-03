import { type ReactNode, useMemo } from "react"
import { LogIn, UserPlus, UserRound } from "lucide-react"
import { useTranslation } from "react-i18next"
import { useDispatch, useSelector } from "react-redux"
import { useLocation, useNavigate } from "react-router"
import {
  CopyableText,
  LanguageSwitch,
  type MobileBottomNavItem,
  SidebarShell,
  ThemeToggle,
  TopBar,
} from "@workspace/app-kit"
import { Button } from "@workspace/ui-components/stable/button"
import type { RootState } from "@/store"
import { logout } from "@/store/authSlice"
import { resetAccess } from "@/store/accessSlice"
import { webNavigationSections } from "@/navigation"

export interface WebAppShellProps {
  children: ReactNode
}

export function WebAppShell({ children }: WebAppShellProps) {
  const dispatch = useDispatch()
  const location = useLocation()
  const navigate = useNavigate()
  const { t } = useTranslation()
  const user = useSelector((state: RootState) => state.auth.user)

  const sections = useMemo(
    () =>
      webNavigationSections.map((section) => ({
        key: section.id,
        label: t(section.labelKey),
        items: section.items.map((item) => {
          const active = item.matcher
            ? item.matcher(location.pathname)
            : location.pathname === item.path

          return {
            key: item.id,
            label: t(item.labelKey),
            href: item.path,
            active,
            icon: item.icon,
            onSelect: () => navigate(item.path),
          }
        }),
      })),
    [location.pathname, navigate, t]
  )
  const mobileNavigationItems = useMemo<MobileBottomNavItem[]>(
    () =>
      webNavigationSections.flatMap((section) =>
        section.items.map((item) => {
          const active = item.matcher
            ? item.matcher(location.pathname)
            : location.pathname === item.path

          return {
            key: item.id,
            label: t(item.labelKey),
            href: item.path,
            active,
            icon: item.icon,
            onSelect: () => navigate(item.path),
          }
        })
      ),
    [location.pathname, navigate, t]
  )

  const currentItem = webNavigationSections
    .flatMap((section) => section.items)
    .find((item) =>
      item.matcher
        ? item.matcher(location.pathname)
        : location.pathname === item.path
    )
  const topBarTitle =
    location.pathname.startsWith("/profile")
      ? t("pages:profile.title")
      : currentItem
        ? t(currentItem.labelKey)
        : t("web.shell.brand.title")

  const handleLogout = () => {
    localStorage.removeItem("token")
    localStorage.removeItem("refreshToken")
    dispatch(resetAccess())
    dispatch(logout())
    navigate("/home", { replace: true })
  }

  const isGuest = !user
  const displayName = user?.name || t("web.shell.account.guestName")
  const displayId = user
    ? user.display_id || user.email || t("web.shell.account.defaultId")
    : t("web.shell.account.guestId")

  const trailing = [
    <LanguageSwitch key="lang" />,
    <ThemeToggle key="theme" />,
    user ? null : (
      <Button key="login" variant="outline" size="sm" onClick={() => navigate("/login")}>
        {t("web.shell.account.login")}
      </Button>
    ),
    user ? null : (
      <Button key="register" size="sm" onClick={() => navigate("/register")}>
        {t("web.shell.account.register")}
      </Button>
    ),
  ].filter(Boolean) as ReactNode[]

  return (
    <SidebarShell
      brandEyebrow={t("web.shell.brand.eyebrow")}
      brandTitle={t("web.shell.brand.title")}
      brandDescription={t("web.shell.brand.description")}
      sections={sections}
      mobileNavigation={
        mobileNavigationItems.length
          ? {
              label: t(
                "web.shell.navigation.mobileLabel"
              ),
              items: mobileNavigationItems,
            }
          : undefined
      }
      header={
        <TopBar
          title={topBarTitle}
          meta={t("web.shell.topbar.meta")}
          trailing={trailing}
        />
      }
      footerAccount={
        {
          avatarAlt: displayName,
          avatarSrc: user?.avatar,
          avatarFallback: isGuest
            ? t("web.shell.account.guestFallback")
            : displayName.charAt(0).toUpperCase(),
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
        }
      }
    >
      {children}
    </SidebarShell>
  )
}
