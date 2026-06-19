import { useMemo } from "react"
import { useQuery } from "@tanstack/react-query"
import { useTranslation } from "react-i18next"
import {
  Badge,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@workspace/ui-components"
import type { BadgeProps } from "@workspace/ui-components"
import { listTeamUsageMembersApi } from "@/api"
import type { TeamUsageRow } from "../types"

export interface TeamMembersDialogProps {
  open: boolean
  row: TeamUsageRow | null
  onOpenChange: (open: boolean) => void
}

function statusVariant(status: string): BadgeProps["variant"] {
  return status === "enabled" ? "default" : "outline"
}

export function TeamMembersDialog({
  open,
  row,
  onOpenChange,
}: TeamMembersDialogProps) {
  const { t } = useTranslation()
  const queryKey = useMemo(
    () => ["admin", "team-usage", "members", row?.id] as const,
    [row?.id]
  )
  const membersQuery = useQuery({
    queryKey,
    queryFn: () => listTeamUsageMembersApi(row?.id ?? ""),
    enabled: open && Boolean(row?.id),
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {t("admin.usage.teamUsage.members.title", {
              team: row?.team_name ?? "",
            })}
          </DialogTitle>
          <DialogDescription>
            {t("admin.usage.teamUsage.members.description", {
              school: row?.school_name ?? "",
            })}
          </DialogDescription>
        </DialogHeader>

        <div className="max-h-96 overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>
                  {t("admin.usage.teamUsage.members.columns.displayId")}
                </TableHead>
                <TableHead>
                  {t("admin.usage.teamUsage.members.columns.displayName")}
                </TableHead>
                <TableHead>
                  {t("admin.usage.teamUsage.members.columns.status")}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {membersQuery.isLoading ? (
                <TableRow>
                  <TableCell colSpan={3}>
                    {t("admin.usage.teamUsage.members.loading")}
                  </TableCell>
                </TableRow>
              ) : null}
              {!membersQuery.isLoading &&
              (membersQuery.data?.length ?? 0) === 0 ? (
                <TableRow>
                  <TableCell colSpan={3}>
                    {t("admin.usage.teamUsage.members.empty")}
                  </TableCell>
                </TableRow>
              ) : null}
              {membersQuery.data?.map((member) => (
                <TableRow key={member.userId}>
                  <TableCell>
                    <span className="font-mono">
                      {member.displayId || member.userId}
                    </span>
                  </TableCell>
                  <TableCell>{member.displayName}</TableCell>
                  <TableCell>
                    <Badge variant={statusVariant(member.status)}>
                      {t(
                        `admin.accounts.appUsers.table.status.${member.status}`
                      )}
                    </Badge>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </DialogContent>
    </Dialog>
  )
}
