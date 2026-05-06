import { AgentChatComposer } from "./components/agent-chat-composer"
import { AgentChatStatus } from "./components/agent-chat-status"
import { AgentChatThread } from "./components/agent-chat-thread"
import { useAgentChatContext } from "./context/use-agent-chat-context"

export default function AgentChatPage() {
  const chat = useAgentChatContext()

  return (
    <div className="flex size-full min-h-0 flex-col overflow-hidden bg-(--app-bg)">
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
