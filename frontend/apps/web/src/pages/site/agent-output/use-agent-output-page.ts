import {
  agentOutputActionKeys,
  agentOutputMetricKeys,
  agentOutputSectionKeys,
  agentOutputStepKeys,
} from "./agent-output-data"
import { useTranslation } from "react-i18next"

export function useAgentOutputPage() {
  const { t } = useTranslation()

  return {
    actions: agentOutputActionKeys.map((key) =>
      t(`agentOutput.actions.${key}`)
    ),
    metrics: agentOutputMetricKeys.map((key) => ({
      label: t(`agentOutput.metrics.${key}.label`),
      value: t(`agentOutput.metrics.${key}.value`),
      description: t(`agentOutput.metrics.${key}.description`),
    })),
    prompt: t("agentOutput.prompt"),
    sections: agentOutputSectionKeys.map((key) => ({
      id: key,
      title: t(`agentOutput.sections.${key}.title`),
      eyebrow: t(`agentOutput.sections.${key}.eyebrow`),
      content: t(`agentOutput.sections.${key}.content`),
      bullets: [0, 1, 2].map((index) =>
        t(`agentOutput.sections.${key}.bullets.${index}`)
      ),
    })),
    steps: agentOutputStepKeys.map((key) => ({
      title: t(`agentOutput.steps.${key}.title`),
      owner: t(`agentOutput.steps.${key}.owner`),
      timing: t(`agentOutput.steps.${key}.timing`),
      detail: t(`agentOutput.steps.${key}.detail`),
    })),
  }
}
