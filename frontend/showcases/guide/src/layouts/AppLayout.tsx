import { useMemo } from "react"
import { Outlet, useLocation, useNavigate } from "react-router"
import {
  Bell,
  BookOpenText,
  Compass,
  FileText,
  LayoutGrid,
  MousePointerClick,
  LayoutPanelTop,
  Palette,
  TableProperties,
} from "lucide-react"
import { LanguageSwitch, ThemeToggle } from "@workspace/app-kit"
import {
  SidebarShell,
  type SidebarShellSection,
} from "@workspace/app-kit/sidebar-shell"
import { TopBar } from "@workspace/app-kit/top-bar"
import { Badge } from "@workspace/ui-components/stable/badge"

const navItems = [
  {
    label: "DataTable 指南",
    path: "/",
    icon: <TableProperties />,
    matcher: (pathname: string) => pathname === "/",
    subItems: [
      { label: "快速预设", href: "/#presets" },
      { label: "实时示例", href: "/#live-demo" },
      { label: "配置说明", href: "/#playbook" },
    ],
  },
  {
    label: "Button 指南",
    path: "/button",
    icon: <MousePointerClick />,
    matcher: (pathname: string) => pathname === "/button",
    subItems: [
      { label: "变体总览", href: "/button#button-variants" },
      { label: "尺寸与布局", href: "/button#button-sizes" },
      { label: "IconButton", href: "/button#icon-button" },
    ],
  },
  {
    label: "表单组件",
    path: "/forms",
    icon: <FileText />,
    matcher: (pathname: string) => pathname === "/forms",
    subItems: [
      { label: "文本输入", href: "/forms#form-text" },
      { label: "选择控件", href: "/forms#form-selection" },
      { label: "日期时间", href: "/forms#form-picker" },
    ],
  },
  {
    label: "反馈浮层",
    path: "/feedback",
    icon: <Bell />,
    matcher: (pathname: string) => pathname === "/feedback",
    subItems: [
      { label: "状态反馈", href: "/feedback#feedback-status" },
      { label: "浮层组件", href: "/feedback#feedback-overlay" },
    ],
  },
  {
    label: "导航菜单",
    path: "/navigation",
    icon: <Compass />,
    matcher: (pathname: string) => pathname === "/navigation",
    subItems: [
      { label: "基础导航", href: "/navigation#nav-basic" },
      { label: "菜单系统", href: "/navigation#nav-menu" },
      { label: "展开结构", href: "/navigation#nav-structure" },
    ],
  },
  {
    label: "展示布局",
    path: "/display",
    icon: <LayoutGrid />,
    matcher: (pathname: string) => pathname === "/display",
    subItems: [
      { label: "基础展示", href: "/display#display-basic" },
      { label: "数据承载", href: "/display#display-data" },
      { label: "媒体布局", href: "/display#display-layout" },
    ],
  },
  {
    label: "Theme 配置",
    path: "/theme",
    icon: <Palette />,
    matcher: (pathname: string) => pathname === "/theme",
    subItems: [
      { label: "语义 token", href: "/theme#semantic-tokens" },
      { label: "双端预览", href: "/theme#preview" },
      { label: "导出 CSS", href: "/theme#export" },
    ],
  },
]

export default function AppLayout() {
  const navigate = useNavigate()
  const location = useLocation()

  const currentSection = useMemo(() => {
    return navItems.find((item) => item.matcher(location.pathname)) ?? navItems[0]
  }, [location.pathname])

  const sections = useMemo<SidebarShellSection[]>(
    () => [
      {
        label: "Guide",
        items: navItems.map((item) => ({
          label: item.label,
          href: item.path,
          active: item.matcher(location.pathname),
          icon: item.icon,
            onSelect: () => navigate(item.path),
            subItems: item.subItems?.map((subItem) => ({
              ...subItem,
              active: false,
              onSelect: () => {
                navigate(item.path)
                window.setTimeout(() => {
                  const hashIndex = subItem.href.indexOf("#")
                  const hash =
                    hashIndex >= 0 ? subItem.href.slice(hashIndex) : ""

                  if (!hash) {
                    return
                  }

                  const anchor = document.querySelector(hash)
                  anchor?.scrollIntoView({ behavior: "smooth", block: "start" })

                  if (window.location.pathname === item.path) {
                    window.history.replaceState(null, "", `${item.path}${hash}`)
                  }
                }, 0)
              },
            })),
        })),
      },
    ],
    [location.pathname, navigate]
  )

  return (
    <div className="h-screen overflow-hidden bg-(--app-bg) text-(--app-text)">
      <div className="flex h-full min-w-0 flex-col overflow-hidden">
        <SidebarShell
          brandEyebrow="Component Docs"
          brandTitle="Guide"
          brandDescription="Admin Shell"
          sections={sections}
          header={
            <TopBar
              title={currentSection.label}
              trailing={[
                <Badge key="badge" variant="outline">
                  No Login
                </Badge>,
                <LanguageSwitch key="lang" />,
                <ThemeToggle key="theme" />,
              ]}
            />
          }
          footerAccount={{
            avatarAlt: "Guide Center",
            avatarFallback: "G",
            displayName: "Guide Center",
            displayId: "datatable@workspace",
            actions: [
              {
                icon: <BookOpenText />,
                label: "组件使用说明",
              },
              {
                icon: <LayoutPanelTop />,
                label: "后台壳架构",
              },
            ],
          }}
        >
          <Outlet />
        </SidebarShell>
      </div>
    </div>
  )
}
