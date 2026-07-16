import { type ReactNode, useCallback, useMemo } from "react"
import { LogIn, School, UserPlus, UserRound } from "lucide-react"
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
import { CommerceAgentLogo } from "@/components/brand"
import { useAuthDialog } from "@/components/auth/AuthDialogProvider"

export interface WebAppShellProps {
  children: ReactNode
}

export function WebAppShell({ children }: WebAppShellProps) {
  const dispatch = useDispatch()
  const location = useLocation()
  const navigate = useNavigate()
  const { t } = useTranslation()
  const user = useSelector((state: RootState) => state.auth.user)
  const { requestLogin } = useAuthDialog()
  const navigateTo = useCallback(
    (path: string) => {
      if (!user && path !== "/commerce" && path !== "/home") {
        requestLogin({ pathname: path })
        return
      }
      navigate(path)
    },
    [navigate, requestLogin, user]
  )

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
            onSelect: () => navigateTo(item.path),
          }
        }),
      })),
    [location.pathname, navigateTo, t]
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
            onSelect: () => navigateTo(item.path),
          }
        })
      ),
    [location.pathname, navigateTo, t]
  )

  const currentItem = webNavigationSections
    .flatMap((section) => section.items)
    .find((item) =>
      item.matcher
        ? item.matcher(location.pathname)
        : location.pathname === item.path
    )
  const topBarTitle = location.pathname.startsWith("/team-school")
    ? t("profile.teamSchoolPage.title")
    : location.pathname.startsWith("/profile")
      ? t("pages:profile.title")
      : currentItem
        ? t(currentItem.labelKey)
        : t("web.shell.brand.title")

  const handleLogout = () => {
    localStorage.removeItem("token")
    localStorage.removeItem("refreshToken")
    dispatch(resetAccess())
    dispatch(logout())
    navigate("/commerce", { replace: true })
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
      <Button
        key="login"
        variant="outline"
        size="sm"
        onClick={() => requestLogin()}
      >
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
      brandLogo={<CommerceAgentLogo className="size-7 rounded-md" size="sm" />}
      brandEyebrow={t("web.shell.brand.eyebrow")}
      brandTitle={t("web.shell.brand.title")}
      brandDescription={t("web.shell.brand.description")}
      onBrandSelect={() => navigateTo("/home")}
      sections={sections}
      mobileNavigation={
        mobileNavigationItems.length
          ? {
              label: t("web.shell.navigation.mobileLabel"),
              items: mobileNavigationItems,
            }
          : undefined
      }
      header={
        <TopBar
          title={topBarTitle}
          meta={t("web.shell.topbar.meta")}
          showSidebarTrigger={false}
          trailing={trailing}
        />
      }
      footerAccount={{
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
              {
                icon: <School />,
                label: t("web.shell.account.actions.teamSchool"),
                onSelect: () => navigate("/team-school"),
              },
            ]
          : [
              {
                icon: <LogIn />,
                label: t("web.shell.account.login"),
                onSelect: () => requestLogin(),
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
