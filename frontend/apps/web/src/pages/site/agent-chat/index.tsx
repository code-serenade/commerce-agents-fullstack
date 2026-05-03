import { AgentChatComposer } from "./components/agent-chat-composer"
import { AgentChatStatus } from "./components/agent-chat-status"
import { AgentChatThread } from "./components/agent-chat-thread"
import { useAgentChatPage } from "./use-agent-chat-page"

export default function AgentChatPage() {
  const chat = useAgentChatPage()

  return (
    <div className="flex h-[calc(100vh-8rem)] min-h-136 w-full flex-col overflow-hidden rounded-lg border border-(--app-border) bg-(--app-bg)">
      <AgentChatStatus error={chat.error} />
      <AgentChatThread messages={chat.messages} />
      <AgentChatComposer
        canSend={chat.canSend}
        disabled={!chat.isLoggedIn}
        input={chat.input}
        isStreaming={chat.isStreaming}
        onInputChange={chat.setInput}
        onSend={chat.sendMessage}
        onStop={chat.stopStreaming}
      />
    </div>
  )
}
