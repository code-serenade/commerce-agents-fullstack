import { useTranslation } from "react-i18next"
import { getHomeRoleClusterAgents } from "../view-model/agent-role-cluster-home-view-model"
import { AgentRoleClusterPanel } from "./agent-role-cluster-panel"

export function AgentRoleClusterHomePanel({
  surface = "card",
}: {
  surface?: "card" | "plain"
}) {
  const { t } = useTranslation()

  return (
    <AgentRoleClusterPanel
      agents={getHomeRoleClusterAgents(t)}
      ariaLabel={t("agentChat.homeRoleCluster.ariaLabel")}
      footer={t("agentChat.homeRoleCluster.footer")}
      itemDetail="summary"
      summary={t("agentChat.homeRoleCluster.summary")}
      surface={surface}
      eyebrow={t("agentChat.homeRoleCluster.eyebrow")}
      title={t("agentChat.homeRoleCluster.title")}
      variant="showcase"
    />
  )
}
