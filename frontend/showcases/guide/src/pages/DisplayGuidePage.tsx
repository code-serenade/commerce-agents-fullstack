import { useState } from "react"
import {
  AspectRatio,
  Avatar,
  AvatarStack,
  Badge,
  Button,
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  Carousel,
  Item,
  Kbd,
  Resizable,
  ScrollArea,
  Separator,
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  Tree,
} from "@workspace/ui-components"
import { DemoBlock, DemoGrid, GuidePage, GuideSection } from "@/components/guide/GuideScaffold"

const POSTER_DATA_URL =
  "data:image/svg+xml;utf8," +
  encodeURIComponent(`
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 675">
      <defs>
        <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stop-color="#1d4ed8" />
          <stop offset="55%" stop-color="#0891b2" />
          <stop offset="100%" stop-color="#14b8a6" />
        </linearGradient>
      </defs>
      <rect width="1200" height="675" fill="url(#g)" />
      <circle cx="930" cy="140" r="120" fill="rgba(255,255,255,0.12)" />
      <circle cx="220" cy="520" r="180" fill="rgba(255,255,255,0.08)" />
      <text x="90" y="260" fill="white" font-size="76" font-family="Arial, sans-serif" font-weight="700">
        Workspace Media
      </text>
      <text x="90" y="330" fill="rgba(255,255,255,0.84)" font-size="32" font-family="Arial, sans-serif">
        AspectRatio demo poster for the guide page
      </text>
    </svg>
  `)

export default function DisplayGuidePage() {
  const [treeValue, setTreeValue] = useState<string[]>(["ops", "ops-api"])

  return (
    <GuidePage
      title="Display & Layout"
      description="这一类组件负责信息承载、媒体展示和可调整布局。内容密度更高，适合后台详情页和资源页。"
      badges={[
        <Badge key="display" variant="outline">
          Display
        </Badge>,
        <Badge key="layout" variant="outline">
          Layout
        </Badge>,
        <Badge key="data" variant="outline">
          Data
        </Badge>,
      ]}
      stats={[
        { label: "Containers", value: "5" },
        { label: "Media & Layout", value: "4" },
        { label: "Data View", value: "3" },
      ]}
    >
      <GuideSection
        id="display-basic"
        title="Basic Display"
        description="卡片、头像、标签、列表项和快捷键提示是后台详情页里最常见的承载组件。"
      >
        <DemoGrid>
          <DemoBlock title="Card / Badge / Kbd" description="基础信息容器和小型状态语义。">
            <Card>
              <CardHeader>
                <CardTitle>Workspace Summary</CardTitle>
                <CardDescription>
                  Compact summary card used on overview pages.
                </CardDescription>
                <CardAction>
                  <Badge variant="outline">Healthy</Badge>
                </CardAction>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <p className="text-sm text-muted-foreground">
                    Last sync completed 3 minutes ago. Audit pipeline is stable.
                  </p>
                  <div className="flex items-center gap-2">
                    <span className="text-sm">Quick open</span>
                    <Kbd keys={["⌘", "K"]} />
                  </div>
                </div>
              </CardContent>
              <CardFooter>
                <div className="flex items-center gap-3">
                  <Button size="sm">Open</Button>
                  <Button size="sm" variant="outline">
                    Dismiss
                  </Button>
                </div>
              </CardFooter>
            </Card>
          </DemoBlock>

          <DemoBlock title="Avatar / Item" description="头像和信息条目组合很适合成员、资源和对象摘要。">
            <div className="space-y-4">
              <div className="flex items-center gap-4">
                <Avatar alt="Alice" fallback="AL" />
                <AvatarStack
                  users={[
                    { alt: "Alice", fallback: "AL" },
                    { alt: "Bob", fallback: "BO" },
                    { alt: "Cathy", fallback: "CA" },
                  ]}
                  overflowCount={2}
                />
              </div>
              <Item
                title="Production CN"
                description="Environment summary with owner, tags and recent status."
                leading={<Avatar alt="Env" fallback="PR" />}
                trailing={<Badge variant="outline">Active</Badge>}
                footer="Owner: Platform Team"
              />
            </div>
          </DemoBlock>
        </DemoGrid>
      </GuideSection>

      <GuideSection
        id="display-data"
        title="Data Presentation"
        description="表格、树结构和滚动区域都属于高密度数据浏览场景。"
      >
        <DemoGrid>
          <DemoBlock title="Table / Separator / ScrollArea" description="经典列表信息块和滚动容器组合。">
            <div className="space-y-4">
              <Table>
                <TableCaption>Recent orders</TableCaption>
                <TableHeader>
                  <TableRow>
                    <TableHead>Order</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Owner</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell>#A-1092</TableCell>
                    <TableCell>Paid</TableCell>
                    <TableCell>Alice</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell>#A-1093</TableCell>
                    <TableCell>Pending</TableCell>
                    <TableCell>Bob</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
              <Separator />
              <ScrollArea
                height={120}
                items={[
                  "Workspace created by Alice",
                  "Billing owner changed to Bob",
                  "API token rotated successfully",
                  "Quarterly audit report exported",
                  "Risk review moved to next Monday",
                ]}
              />
            </div>
          </DemoBlock>

          <DemoBlock title="Tree" description="适合目录、权限树、区域树和多级资源选择。">
            <div className="rounded-2xl border border-border/60 p-3">
              <Tree
                data={[
                  {
                    id: "ops",
                    label: "Operations",
                    children: [
                      { id: "ops-api", label: "API" },
                      { id: "ops-dashboard", label: "Dashboard" },
                    ],
                  },
                  {
                    id: "sales",
                    label: "Sales",
                    children: [
                      { id: "sales-cn", label: "China" },
                      { id: "sales-jp", label: "Japan" },
                    ],
                  },
                ]}
                value={treeValue}
                onValueChange={setTreeValue}
                defaultExpandedIds={["ops", "sales"]}
              />
            </div>
          </DemoBlock>
        </DemoGrid>
      </GuideSection>

      <GuideSection
        id="display-layout"
        title="Media & Layout"
        description="媒体卡片、走马灯和可调整布局通常在 dashboard 或内容页出现。"
      >
        <DemoGrid>
          <DemoBlock title="AspectRatio / Carousel" description="媒体容器和轮播展示。">
            <div className="space-y-4">
              <AspectRatio
                src={POSTER_DATA_URL}
                alt="Guide poster"
                ratio={16 / 9}
              />
              <Carousel
                slides={[
                  "Quarterly report cover",
                  "Campaign gallery",
                  "Onboarding snapshots",
                ]}
              />
            </div>
          </DemoBlock>

          <DemoBlock title="Resizable" description="双栏工作区、预览对比和 inspector 面板都适合这个布局。">
            <div className="h-[260px]">
              <Resizable
                left={
                  <div className="space-y-3">
                    <p className="text-sm font-medium">Editor</p>
                    <p className="text-sm text-muted-foreground">
                      Editable content and filters stay on the left side.
                    </p>
                  </div>
                }
                right={
                  <div className="space-y-3">
                    <p className="text-sm font-medium">Preview</p>
                    <p className="text-sm text-muted-foreground">
                      Real-time preview, logs or diff panel can live here.
                    </p>
                  </div>
                }
              />
            </div>
          </DemoBlock>
        </DemoGrid>
      </GuideSection>
    </GuidePage>
  )
}
