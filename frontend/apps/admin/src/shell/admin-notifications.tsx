import {
  Check,
  CircleX,
  FileText,
  MessageCircleReply,
  MessageSquareMore,
} from "lucide-react"
import type { TFunction } from "i18next"
import type { NotificationDropdownItem } from "@workspace/app-kit"

function avatarClassName(tone: "lime" | "cyan" | "violet") {
  if (tone === "lime") {
    return "bg-linear-to-br from-lime-300 to-emerald-400 text-white"
  }

  if (tone === "cyan") {
    return "bg-linear-to-br from-lime-300 to-cyan-400 text-white"
  }

  return "bg-linear-to-br from-indigo-600 to-rose-500 text-white"
}

function iconButtonClassName(tone: "default" | "destructive") {
  if (tone === "destructive") {
    return "size-5 text-rose-500"
  }

  return "size-5 text-foreground"
}

export function getAdminNotifications(
  t: TFunction
): NotificationDropdownItem[] {
  return [
    {
      id: "weekly-reports",
      title: t("admin.shell.notifications.items.weeklyReports.title"),
      description: t(
        "admin.shell.notifications.items.weeklyReports.description"
      ),
      timeLabel: t("admin.shell.notifications.items.weeklyReports.time"),
      leading: (
        <span
          className={`flex size-12 items-center justify-center rounded-full text-lg font-semibold ${avatarClassName("lime")}`}
        >
          VB
        </span>
      ),
      action: {
        label: t("admin.shell.notifications.actions.dismiss"),
        icon: (
          <CircleX
            className={iconButtonClassName("destructive")}
            strokeWidth={2.2}
          />
        ),
      },
    },
    {
      id: "reply",
      title: t("admin.shell.notifications.items.reply.title"),
      description: t("admin.shell.notifications.items.reply.description"),
      timeLabel: t("admin.shell.notifications.items.reply.time"),
      unread: true,
      leading: (
        <span
          className={`flex size-12 items-center justify-center rounded-full ${avatarClassName("cyan")}`}
        >
          <MessageCircleReply className="size-5" strokeWidth={2.2} />
        </span>
      ),
      action: {
        label: t("admin.shell.notifications.actions.markRead"),
        icon: (
          <Check className={iconButtonClassName("default")} strokeWidth={2.4} />
        ),
      },
    },
    {
      id: "comment",
      title: t("admin.shell.notifications.items.comment.title"),
      description: t("admin.shell.notifications.items.comment.description"),
      timeLabel: t("admin.shell.notifications.items.comment.time"),
      unread: true,
      leading: (
        <span
          className={`flex size-12 items-center justify-center rounded-full ${avatarClassName("cyan")}`}
        >
          <MessageSquareMore className="size-5" strokeWidth={2.2} />
        </span>
      ),
      action: {
        label: t("admin.shell.notifications.actions.markRead"),
        icon: (
          <Check className={iconButtonClassName("default")} strokeWidth={2.4} />
        ),
      },
    },
    {
      id: "task-reminder",
      title: t("admin.shell.notifications.items.taskReminder.title"),
      description: t(
        "admin.shell.notifications.items.taskReminder.description"
      ),
      timeLabel: t("admin.shell.notifications.items.taskReminder.time"),
      unread: true,
      leading: (
        <span
          className={`flex size-12 items-center justify-center rounded-full ${avatarClassName("violet")}`}
        >
          <FileText className="size-5" strokeWidth={2.1} />
        </span>
      ),
      action: {
        label: t("admin.shell.notifications.actions.markRead"),
        icon: (
          <Check className={iconButtonClassName("default")} strokeWidth={2.4} />
        ),
      },
    },
  ]
}
