import { useMemo } from "react"
import type { TFunction } from "i18next"
import { useTranslation } from "react-i18next"
import { useSelector } from "react-redux"
import { useLocation, useNavigate } from "react-router"
import type { SidebarShellSection } from "@workspace/app-kit/sidebar-shell"
import type { RootState } from "@/store"
import {
  navigationSections,
  type NavigationItemConfig,
  type NavigationSectionConfig,
  type NavigationSubItemConfig,
} from "./menu-config"

interface CurrentNavigationItem {
  label: string
  path: string
}

function matchesNavigationSubItem(
  subItem: NavigationSubItemConfig,
  pathname: string
) {
  const matcher =
    subItem.matcher ??
    ((currentPathname: string) => currentPathname === subItem.href)

  return matcher(pathname)
}

function isVisibleSubItem(
  subItem: NavigationSubItemConfig,
  visibleCodes: Set<string>
) {
  return visibleCodes.has(subItem.permissionCode)
}

function findCurrentItem(
  sections: NavigationSectionConfig[],
  pathname: string,
  t: TFunction
): CurrentNavigationItem | null {
  for (const section of sections) {
    for (const item of section.items) {
      if (item.subItems?.length) {
        for (const subItem of item.subItems) {
          if (matchesNavigationSubItem(subItem, pathname)) {
            return {
              label: translateNavigationLabel(t, subItem),
              path: subItem.href,
            }
          }
        }
      }

      if (!item.path) {
        continue
      }

      if (pathname === item.path) {
        return {
          label: translateNavigationLabel(t, item),
          path: item.path,
        }
      }
    }
  }

  return null
}

function getDefaultItem(
  sections: NavigationSectionConfig[],
  t: TFunction
): CurrentNavigationItem | null {
  for (const section of sections) {
    for (const item of section.items) {
      const firstSubItem = item.subItems?.[0]
      if (firstSubItem) {
        return {
          label: translateNavigationLabel(t, firstSubItem),
          path: firstSubItem.href,
        }
      }

      if (item.path) {
        return {
          label: translateNavigationLabel(t, item),
          path: item.path,
        }
      }
    }
  }

  return null
}

function toSidebarSubItems(
  subItems: NavigationSubItemConfig[] | undefined,
  pathname: string,
  navigate: ReturnType<typeof useNavigate>,
  t: TFunction
) {
  return subItems?.map((subItem) => ({
    key: subItem.id,
    label: translateNavigationLabel(t, subItem),
    href: subItem.href,
    active: matchesNavigationSubItem(subItem, pathname),
    onSelect: () => navigate(subItem.href),
  }))
}

function translateNavigationLabel(
  t: TFunction,
  item: { labelKey: string }
) {
  return t(item.labelKey)
}

export function useAdminNavigation() {
  const navigate = useNavigate()
  const location = useLocation()
  const { t } = useTranslation()
  const permissionCodes = useSelector(
    (state: RootState) => state.adminAccess.permissionCodes
  )
  const visibleCodes = useMemo(
    () => new Set(permissionCodes),
    [permissionCodes]
  )

  const visibleSections = useMemo<NavigationSectionConfig[]>(
    () =>
      navigationSections
        .map((section) => ({
          ...section,
          items: section.items
            .map((item) => {
              const visibleSubItems = item.subItems?.filter((subItem) =>
                isVisibleSubItem(subItem, visibleCodes)
              )

              if (visibleSubItems?.length) {
                return {
                  ...item,
                  subItems: visibleSubItems,
                }
              }

              if (
                item.permissionCode &&
                visibleCodes.has(item.permissionCode)
              ) {
                return item
              }

              return null
            })
            .filter((item): item is NavigationItemConfig => Boolean(item)),
        }))
        .filter((section) => section.items.length > 0),
    [visibleCodes]
  )

  const currentItem = useMemo(
    () => findCurrentItem(visibleSections, location.pathname, t),
    [location.pathname, t, visibleSections]
  )

  const defaultItem = useMemo(
    () => getDefaultItem(visibleSections, t),
    [t, visibleSections]
  )

  const sections = useMemo<SidebarShellSection[]>(
    () =>
      visibleSections.map((section) => ({
        key: section.id,
        label: translateNavigationLabel(t, section),
        items: section.items.map((item) => {
          return {
            key: item.id,
            label: translateNavigationLabel(t, item),
            href: item.path,
            active: item.path ? location.pathname === item.path : false,
            icon: item.icon,
            onSelect: item.path ? () => navigate(item.path!) : undefined,
            subItems: toSidebarSubItems(
              item.subItems,
              location.pathname,
              navigate,
              t
            ),
          }
        }),
      })),
    [location.pathname, navigate, t, visibleSections]
  )

  const labelsByPath = useMemo<Record<string, string>>(() => {
    const entries: Array<[string, string]> = []

    for (const section of visibleSections) {
      for (const item of section.items) {
        if (item.path) {
          entries.push([item.path, translateNavigationLabel(t, item)])
        }

        for (const subItem of item.subItems ?? []) {
          entries.push([subItem.href, translateNavigationLabel(t, subItem)])
        }
      }
    }

    return Object.fromEntries(entries)
  }, [t, visibleSections])

  return {
    currentItem,
    defaultPath: defaultItem?.path ?? null,
    hasVisibleMenus: visibleSections.length > 0,
    isLoading: false,
    labelsByPath,
    sections,
  }
}
