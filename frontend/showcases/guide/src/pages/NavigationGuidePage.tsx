import { useState } from "react"
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
  Badge,
  Breadcrumb,
  Button,
  ButtonGroup,
  Collapsible,
  Command,
  ContextMenu,
  Kbd,
  Menubar,
  NavigationMenu,
  Pagination,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  toast,
} from "@workspace/ui-components"
import { DemoBlock, DemoGrid, GuidePage, GuideSection } from "@/components/guide/GuideScaffold"

export default function NavigationGuidePage() {
  const [activeTab, setActiveTab] = useState("overview")
  const [page, setPage] = useState(6)
  const [collapsibleOpen, setCollapsibleOpen] = useState(true)

  return (
    <GuidePage
      title="Navigation & Menus"
      description="这类组件负责页面切换、结构定位和命令入口。示例里包含静态导航、右键菜单和命令面板。"
      badges={[
        <Badge key="tabs" variant="outline">
          Tabs
        </Badge>,
        <Badge key="menu" variant="outline">
          Menus
        </Badge>,
        <Badge key="command" variant="outline">
          Command
        </Badge>,
      ]}
      stats={[
        { label: "Navigation", value: "4" },
        { label: "Menu", value: "3" },
        { label: "Structure", value: "3" },
      ]}
    >
      <GuideSection
        id="nav-basic"
        title="Basic Navigation"
        description="最常见的定位组件：Breadcrumb、Tabs、Pagination。"
      >
        <DemoGrid>
          <DemoBlock title="Breadcrumb / ButtonGroup" description="路径定位和紧凑动作分组。">
            <div className="space-y-5">
              <Breadcrumb
                items={[
                  { label: "Workspace", href: "#" },
                  { label: "Billing", href: "#" },
                  { label: "Invoices" },
                ]}
              />
              <ButtonGroup
                items={[
                  <span key="day">Today</span>,
                  <span key="week">This Week</span>,
                  <span key="month">This Month</span>,
                ]}
              />
            </div>
          </DemoBlock>

          <DemoBlock title="Tabs / Pagination" description="页签切换和列表翻页。">
            <div className="space-y-5">
              <Tabs value={activeTab} onValueChange={setActiveTab}>
                <TabsList>
                  <TabsTrigger value="overview">Overview</TabsTrigger>
                  <TabsTrigger value="members">Members</TabsTrigger>
                  <TabsTrigger value="audit">Audit</TabsTrigger>
                </TabsList>
                <TabsContent value="overview">
                  <p className="text-sm text-muted-foreground">
                    Overview tab keeps summary, metrics and shortcuts.
                  </p>
                </TabsContent>
                <TabsContent value="members">
                  <p className="text-sm text-muted-foreground">
                    Members tab focuses on invites and permission rules.
                  </p>
                </TabsContent>
                <TabsContent value="audit">
                  <p className="text-sm text-muted-foreground">
                    Audit tab keeps immutable logs and export actions.
                  </p>
                </TabsContent>
              </Tabs>

              <Pagination
                page={page}
                totalPages={18}
                onPageChange={setPage}
                jumpPageCount={4}
              />
            </div>
          </DemoBlock>
        </DemoGrid>
      </GuideSection>

      <GuideSection
        id="nav-menu"
        title="Menus"
        description="顶栏菜单、右键菜单和站点级导航菜单。"
      >
        <DemoGrid>
          <DemoBlock title="NavigationMenu / Menubar" description="前者偏站点导航，后者偏应用操作。">
            <div className="space-y-5">
              <NavigationMenu
                items={[
                  { type: "link", label: "Dashboard", href: "#" },
                  {
                    type: "group",
                    label: "Resources",
                    links: [
                      { label: "Guides", href: "#" },
                      { label: "Changelog", href: "#" },
                      { label: "API Reference", href: "#" },
                    ],
                  },
                ]}
              />

              <Menubar
                menus={[
                  {
                    label: "File",
                    items: [
                      {
                        label: "New Project",
                        onSelect: () => toast("Create project"),
                      },
                      {
                        label: "Export Data",
                        onSelect: () => toast("Export queued"),
                      },
                    ],
                  },
                  {
                    label: "Danger",
                    items: [
                      {
                        label: "Delete Workspace",
                        variant: "destructive",
                        onSelect: () => toast("Danger action triggered"),
                      },
                    ],
                  },
                ]}
              />
            </div>
          </DemoBlock>

          <DemoBlock title="ContextMenu / Command" description="右键入口适合局部操作，Command 适合全局检索和跳转。">
            <div className="space-y-5">
              <ContextMenu
                trigger="Right click this panel"
                items={[
                  { label: "Rename", onSelect: () => toast("Rename") },
                  { label: "Duplicate", onSelect: () => toast("Duplicate") },
                  {
                    label: "Delete",
                    variant: "destructive",
                    onSelect: () => toast("Delete"),
                  },
                ]}
              />

              <div className="rounded-2xl border border-border/60">
                <Command
                  sections={[
                    {
                      heading: "Pages",
                      items: [
                        {
                          label: "Open Billing",
                          value: "billing",
                          shortcut: <Kbd keys={["G", "B"]} />,
                        },
                        {
                          label: "Open Members",
                          value: "members",
                          shortcut: <Kbd keys={["G", "M"]} />,
                        },
                      ],
                    },
                    {
                      heading: "Actions",
                      items: [
                        { label: "Create Workspace", value: "create" },
                        { label: "Archive Project", value: "archive" },
                      ],
                    },
                  ]}
                  onSelect={(value) => toast(`Command selected: ${value}`)}
                />
              </div>
            </div>
          </DemoBlock>
        </DemoGrid>
      </GuideSection>

      <GuideSection
        id="nav-structure"
        title="Expandable Structure"
        description="适合分层内容或按需展开的信息块。"
      >
        <DemoGrid>
          <DemoBlock title="Accordion" description="多段内容折叠展示，常见于 FAQ 或设置分组。">
            <Accordion type="single" collapsible defaultValue="api">
              <AccordionItem value="api">
                <AccordionTrigger>API Access</AccordionTrigger>
                <AccordionContent>
                  Tokens, scopes and callback URLs stay in this block.
                </AccordionContent>
              </AccordionItem>
              <AccordionItem value="notify">
                <AccordionTrigger>Notifications</AccordionTrigger>
                <AccordionContent>
                  Email, SMS and webhook policies live here.
                </AccordionContent>
              </AccordionItem>
            </Accordion>
          </DemoBlock>

          <DemoBlock title="Collapsible" description="比 Accordion 更适合页面内单块信息的展开和收起。">
            <div className="space-y-3">
              <Collapsible
                open={collapsibleOpen}
                onOpenChange={setCollapsibleOpen}
                triggerLabel={
                  <Button variant="outline">
                    {collapsibleOpen ? "Hide details" : "Show details"}
                  </Button>
                }
              >
                <div className="rounded-xl border border-border/60 p-3 text-sm text-muted-foreground">
                  Release branch, rollback version and owner checklist are grouped
                  in this collapsible area.
                </div>
              </Collapsible>
            </div>
          </DemoBlock>
        </DemoGrid>
      </GuideSection>
    </GuidePage>
  )
}
