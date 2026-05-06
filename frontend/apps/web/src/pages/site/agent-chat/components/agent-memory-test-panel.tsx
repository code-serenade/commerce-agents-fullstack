import { RotateCcw } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Badge } from "@workspace/ui-components/stable/badge"
import { Button } from "@workspace/ui-components/stable/button"
import type { AgentMemoryTestState } from "../types"

export interface AgentMemoryTestPanelProps {
  disabled: boolean
  isStreaming: boolean
  memoryTest: AgentMemoryTestState
  onPrepareRecall: () => void
  onPrepareSave: () => void
  onResetKey: () => void
}

export function AgentMemoryTestPanel({
  disabled,
  isStreaming,
  memoryTest,
  onPrepareRecall,
  onPrepareSave,
  onResetKey,
}: AgentMemoryTestPanelProps) {
  const { t } = useTranslation()
  const actionDisabled = disabled || isStreaming

  return (
    <section className="shrink-0 border-b border-(--app-border) bg-(--app-surface) px-4 py-3 sm:px-6">
      <div className="mx-auto flex w-full max-w-3xl flex-wrap items-center gap-2">
        <Badge variant="secondary">{t("agentChat.memoryTest.title")}</Badge>
        <code className="rounded-md border border-(--app-border) bg-(--app-muted-surface) px-2 py-1 text-xs text-(--app-text)">
          {memoryTest.testKey}
        </code>
        <div className="flex flex-wrap items-center gap-1 text-xs text-(--app-muted-text)">
          <span>
            {t("agentChat.memoryTest.loaded", {
              count: memoryTest.loadedCount,
            })}
          </span>
          <span>/</span>
          <span>
            {t("agentChat.memoryTest.persisted", {
              count: memoryTest.persistedCount,
            })}
          </span>
          <span>/</span>
          <span>
            {t("agentChat.memoryTest.compacted", {
              count: memoryTest.compactedCount,
            })}
          </span>
        </div>
        <div className="ml-auto flex flex-wrap items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={actionDisabled}
            onClick={onPrepareSave}
          >
            {t("agentChat.memoryTest.savePrompt")}
          </Button>
          <Button
            variant="outline"
            size="sm"
            disabled={actionDisabled}
            onClick={onPrepareRecall}
          >
            {t("agentChat.memoryTest.recallPrompt")}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            disabled={actionDisabled}
            onClick={onResetKey}
          >
            <RotateCcw className="size-4" />
            {t("agentChat.memoryTest.reset")}
          </Button>
        </div>
      </div>
    </section>
  )
}
