import {
  Bot,
  Coffee,
  Database,
  Loader2,
  Megaphone,
  Plus,
  Send,
  ShoppingBag,
  Square,
} from "lucide-react"
import { type KeyboardEvent } from "react"
import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components/stable/button"
import type { AgentLlmConfigResponse } from "@/api"

const SAMPLE_PROMPTS = [
  {
    label: "五一咖啡转化",
    icon: Coffee,
    prompt:
      "我是做咖啡运营的，主要营销平台是抖音，现在马上五一了，可以帮我写一个运营方案提升我的转化率，包含千川投放、预算、创意和执行计划",
  },
  {
    label: "千川投放复盘",
    icon: Megaphone,
    prompt:
      "帮我复盘一个抖音直播间千川投放问题：点击率可以但下单转化低，请按数据诊断、预算、创意、投放和运营执行拆成多智能体任务",
  },
]

const CONTEXT_CHIPS = [
  {
    label: "咖啡商品组",
    icon: ShoppingBag,
    title: "后续接商品看板和商品知识",
  },
  {
    label: "五一营销活动",
    icon: Megaphone,
    title: "后续接营销活动策划和执行跟踪",
  },
  {
    label: "抖音/千川数据",
    icon: Database,
    title: "后续接平台连接、数据导入和投放数据",
  },
]

export interface AgentChatComposerProps {
  canSend: boolean
  disabled: boolean
  input: string
  isStreaming: boolean
  llmConfig: AgentLlmConfigResponse | null
  onInputChange: (value: string) => void
  onModelChange: (value: string | null) => void
  onSend: () => void
  onStop: () => void
  selectedModelName: string | null
}

export function AgentChatComposer({
  canSend,
  disabled,
  input,
  isStreaming,
  llmConfig,
  onInputChange,
  onModelChange,
  onSend,
  onStop,
  selectedModelName,
}: AgentChatComposerProps) {
  const { t } = useTranslation()
  const selectedModel =
    llmConfig?.items.find((item) => item.name === selectedModelName) ??
    llmConfig?.items.find((item) => item.selected) ??
    llmConfig?.items[0]

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key !== "Enter" || event.shiftKey || event.nativeEvent.isComposing) return
    event.preventDefault()
    if (canSend) onSend()
  }

  return (
    <footer className="shrink-0 bg-background/95 p-3 sm:px-5">
      <div
        className="mx-auto flex w-full max-w-3xl flex-col gap-3 rounded-[1.65rem] border border-border bg-[#f1f1ef] p-3 shadow-xl shadow-black/10 dark:bg-card"
        onKeyDown={handleKeyDown}
      >
        <div className="flex flex-wrap gap-2">
          {SAMPLE_PROMPTS.map((sample) => {
            const Icon = sample.icon
            return (
              <button
                key={sample.label}
                type="button"
                disabled={disabled || isStreaming}
                onClick={() => onInputChange(sample.prompt)}
                className="flex items-center gap-2 rounded-full border border-border bg-background px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
                title={sample.prompt}
              >
                <Icon className="size-3.5" />
                {sample.label}
              </button>
            )
          })}
          <span className="hidden h-7 w-px bg-border sm:block" />
          {CONTEXT_CHIPS.map((chip) => {
            const Icon = chip.icon
            return (
              <button
                key={chip.label}
                type="button"
                disabled={disabled || isStreaming}
                className="flex items-center gap-2 rounded-full border border-border bg-background/70 px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
                title={chip.title}
              >
                <Icon className="size-3.5" />
                {chip.label}
              </button>
            )
          })}
        </div>
        <textarea
          value={input}
          onChange={(event) => onInputChange(event.target.value)}
          placeholder={t(
            disabled
              ? "agentChat.composer.loginPlaceholder"
              : "agentChat.composer.placeholder"
          )}
          disabled={disabled || isStreaming}
          rows={2}
          className="max-h-32 min-h-14 w-full resize-none rounded-md border-0 bg-transparent px-2 py-2 text-sm/6 outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-60"
        />
        <div className="flex min-w-0 items-center justify-between gap-2 rounded-full bg-background px-2 py-2 shadow-sm">
          <div className="flex min-w-0 flex-1 items-center gap-2 overflow-hidden">
            <button
              type="button"
              className="flex size-10 shrink-0 items-center justify-center rounded-full border border-border bg-card text-muted-foreground transition-colors hover:text-foreground"
              title="添加文件"
            >
              <Plus className="size-5" />
            </button>
            <button
              type="button"
              className="flex min-w-0 max-w-[14rem] shrink items-center gap-2 rounded-full border border-border bg-card px-3 py-2 text-sm text-foreground transition-colors hover:bg-muted"
              title="切换运营智能体"
            >
              <Bot className="size-4 shrink-0 text-blue-500" />
              <span className="truncate">Commerce 运营智能体</span>
            </button>
            {llmConfig?.items.length ? (
              <label className="flex min-w-[7.5rem] shrink-0 items-center gap-2 rounded-full border border-border bg-card px-3 py-2 text-xs text-foreground">
                <span className="hidden text-xs text-muted-foreground sm:inline">模型</span>
                <select
                  value={selectedModel?.name ?? ""}
                  onChange={(event) => onModelChange(event.target.value || null)}
                  disabled={disabled || isStreaming}
                  className="min-w-0 max-w-32 bg-transparent text-xs outline-none disabled:cursor-not-allowed"
                  title={selectedModel ? `${selectedModel.baseUrl} · ${selectedModel.model}` : "选择模型"}
                >
                  {llmConfig.items.map((item) => (
                    <option key={item.name} value={item.name}>
                      {item.model}
                    </option>
                  ))}
                </select>
              </label>
            ) : null}
            <p className="hidden shrink-0 text-xs text-muted-foreground min-[1700px]:block">
              {t("agentChat.composer.hint")}
            </p>
          </div>
          {isStreaming ? (
            <Button variant="outline" size="icon" onClick={onStop}>
              <Square className="size-4" />
            </Button>
          ) : (
            <button
              type="button"
              disabled={!canSend}
              onClick={onSend}
              className="flex size-9 shrink-0 items-center justify-center rounded-full bg-foreground text-background transition-colors hover:bg-foreground/90 disabled:cursor-not-allowed disabled:opacity-45"
              title={t("agentChat.composer.send")}
            >
              {canSend ? <Send className="size-4" /> : <Loader2 className="size-4" />}
            </button>
          )}
        </div>
      </div>
    </footer>
  )
}
