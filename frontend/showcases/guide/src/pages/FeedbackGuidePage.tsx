import { useState } from "react"
import {
  Alert,
  AlertDialog,
  Badge,
  Button,
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  Drawer,
  HoverCard,
  Popover,
  PopoverContent,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  PopoverTrigger,
  Progress,
  Sheet,
  Skeleton,
  Spinner,
  toast,
} from "@workspace/ui-components"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@workspace/app-kit"
import { DemoBlock, DemoGrid, GuidePage, GuideSection } from "@/components/guide/GuideScaffold"

export default function FeedbackGuidePage() {
  const [dialogOpen, setDialogOpen] = useState(false)
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [sheetOpen, setSheetOpen] = useState(false)
  const [alertDialogOpen, setAlertDialogOpen] = useState(false)
  const [popoverOpen, setPopoverOpen] = useState(false)

  return (
    <GuidePage
      title="Feedback & Overlay"
      description="这一类组件主要处理状态反馈、提示层和确认交互。页面里同时覆盖静态样式和需要 open state 的受控弹层。"
      badges={[
        <Badge key="feedback" variant="outline">
          Feedback
        </Badge>,
        <Badge key="overlay" variant="outline">
          Overlay
        </Badge>,
        <Badge key="confirm" variant="outline">
          Confirm
        </Badge>,
      ]}
      stats={[
        { label: "Inline Status", value: "5" },
        { label: "Overlay", value: "6" },
        { label: "Toast", value: "1" },
      ]}
    >
      <GuideSection
        id="feedback-status"
        title="Inline Feedback"
        description="适合页面内直接展示状态、进度和 loading 状态。"
      >
        <DemoGrid>
          <DemoBlock title="Alert Variants" description="默认、成功、警告和错误状态都在这一层完成。">
            <div className="space-y-3">
              <Alert
                title="Default Notice"
                description="Use this for neutral explanations and inline notices."
              />
              <Alert
                variant="success"
                title="Deployment Succeeded"
                description="The latest release was published to production."
              />
              <Alert
                variant="warning"
                title="Pending Review"
                description="This workflow still needs finance confirmation."
              />
              <Alert
                variant="error"
                title="Sync Failed"
                description="Retry after checking the webhook signature."
              />
            </div>
          </DemoBlock>

          <DemoBlock title="Progress / Spinner / Skeleton" description="基础 loading 反馈和占位组件。">
            <div className="space-y-5">
              <div className="space-y-2">
                <Progress value={68} />
                <p className="text-xs text-muted-foreground">
                  Upload progress: 68%
                </p>
              </div>
              <div className="flex items-center gap-4">
                <Spinner size="sm" />
                <Spinner />
                <Spinner size="lg" />
              </div>
              <div className="space-y-3">
                <Skeleton size="sm" />
                <Skeleton />
                <Skeleton size="lg" />
              </div>
            </div>
          </DemoBlock>
        </DemoGrid>
      </GuideSection>

      <GuideSection
        id="feedback-overlay"
        title="Overlay Components"
        description="轻提示和大弹层放在同一页，业务里可以按信息密度决定用哪一种。"
      >
        <DemoGrid>
          <DemoBlock title="Tooltip / HoverCard / Popover" description="从最轻的提示到可承载操作的浮层。">
            <div className="flex flex-wrap items-center gap-3">
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger>
                    <Button variant="outline">Tooltip Trigger</Button>
                  </TooltipTrigger>
                  <TooltipContent>Shows a short hint on hover.</TooltipContent>
                </Tooltip>
              </TooltipProvider>

              <HoverCard
                triggerLabel={<Button variant="outline">HoverCard Trigger</Button>}
                title="Release Window"
                description="Weekdays only, 09:00 to 18:00."
              />

              <Popover open={popoverOpen} onOpenChange={setPopoverOpen}>
                <PopoverTrigger>
                  <Button variant="outline">Popover Trigger</Button>
                </PopoverTrigger>
                <PopoverContent>
                  <PopoverHeader>
                    <PopoverTitle>Quick Action</PopoverTitle>
                    <PopoverDescription>
                      Popover 适合短说明和紧凑操作。
                    </PopoverDescription>
                  </PopoverHeader>
                </PopoverContent>
              </Popover>
            </div>
          </DemoBlock>

          <DemoBlock title="Toast" description="全局通知已经在 app 根节点挂好了 Provider，这里直接触发即可。">
            <div className="flex flex-wrap items-center gap-3">
              <Button onClick={() => toast("Workspace synced successfully.")}>
                Trigger Toast
              </Button>
              <Button
                variant="outline"
                onClick={() => toast("Build queued, waiting for runner.")}
              >
                Secondary Toast
              </Button>
            </div>
          </DemoBlock>
        </DemoGrid>

        <DemoGrid>
          <DemoBlock title="Dialog" description="适合信息密度高、需要多按钮确认的模态弹窗。">
            <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
              <DialogTrigger>
                <Button>Open Dialog</Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Review Deployment Plan</DialogTitle>
                  <DialogDescription>
                    Confirm target environment, owners and rollback strategy.
                  </DialogDescription>
                </DialogHeader>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <p>Environment: Production CN</p>
                  <p>Approvers: Alice, Cathy</p>
                  <p>Rollback: Available within 5 minutes</p>
                </div>
                <DialogFooter>
                  <DialogClose>
                    <Button variant="outline">Cancel</Button>
                  </DialogClose>
                  <DialogClose>
                    <Button>Confirm</Button>
                  </DialogClose>
                </DialogFooter>
              </DialogContent>
            </Dialog>
          </DemoBlock>

          <DemoBlock title="Drawer / Sheet / AlertDialog" description="移动端抽屉、侧边面板和危险确认弹窗。">
            <div className="flex flex-wrap items-center gap-3">
              <Drawer
                open={drawerOpen}
                onOpenChange={setDrawerOpen}
                triggerLabel={<Button variant="outline">Open Drawer</Button>}
                title="Mobile Actions"
                description="A compact action list for narrow screens."
              >
                <div className="px-4 text-sm text-muted-foreground">
                  Sync invoices, view logs and export reports.
                </div>
              </Drawer>

              <Sheet
                open={sheetOpen}
                onOpenChange={setSheetOpen}
                triggerLabel={<Button variant="outline">Open Sheet</Button>}
                title="Inspector"
                description="Right-side panel for settings and quick edits."
                side="right"
              >
                <div className="mt-4 text-sm text-muted-foreground">
                  Editable summary, side notes and audit metadata.
                </div>
              </Sheet>

              <AlertDialog
                open={alertDialogOpen}
                onOpenChange={setAlertDialogOpen}
                triggerLabel={<Button variant="destructive">Danger Action</Button>}
                title="Delete Workspace"
                description="This action cannot be undone. All related API keys will be revoked."
                confirmLabel="Delete"
                onConfirm={() => toast("Workspace deleted.")}
              />
            </div>
          </DemoBlock>
        </DemoGrid>
      </GuideSection>
    </GuidePage>
  )
}
