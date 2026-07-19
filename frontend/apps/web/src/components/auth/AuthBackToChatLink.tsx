import { Link } from "react-router"
import { useTranslation } from "react-i18next"
import { ArrowLeft } from "lucide-react"

const backToChatClassName =
  "absolute top-6 left-5 z-30 inline-flex h-9 cursor-pointer items-center gap-2 rounded-xl border border-[rgba(209,220,240,0.96)] bg-white/80 px-3 text-sm font-bold text-[#475467] shadow-[0_12px_28px_rgba(38,76,160,0.08)] backdrop-blur-xl transition hover:bg-white/90 hover:text-[#2758ff] sm:top-10 sm:left-12 dark:border-[rgba(159,179,217,0.22)] dark:bg-[rgba(11,18,38,0.58)] dark:text-[#d6e2ff] dark:shadow-[0_16px_34px_rgba(0,0,0,0.22)] dark:hover:bg-[rgba(16,25,50,0.78)] dark:hover:text-[#76a2ff]"

export function AuthBackToChatLink() {
  const { t } = useTranslation()

  return (
    <Link to="/commerce" className={backToChatClassName}>
      <ArrowLeft aria-hidden="true" className="size-4" strokeWidth={2.2} />
      <span>{t("login.footer.backToChat")}</span>
    </Link>
  )
}
