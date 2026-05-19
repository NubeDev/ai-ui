import "@openuidev/react-ui/components.css";
import { openAIAdapter, openAIMessageFormat } from "@openuidev/react-headless";
import { FullScreen } from "@openuidev/react-ui";
import { openuiLibrary } from "@openuidev/react-ui/genui-lib";
import type { ReactNode } from "react";

/**
 * Props accepted by the {@link AiUi} component.
 *
 * `endpoint` defaults to `/api/chat`, matching `router::chat_routes` on the
 * Rust side. Everything else passes through to OpenUI's `<FullScreen>`.
 */
export interface AiUiProps {
  /** Where to POST chat requests. Default `/api/chat`. */
  endpoint?: string;
  /** Display name shown in the chat header. */
  agentName?: string;
  /** OpenUI component library. Defaults to the stock genui-lib. */
  componentLibrary?: unknown;
  /** Optional opener prompts shown in the empty chat. */
  conversationStarters?: {
    variant?: "short" | "long";
    options: Array<{ displayText: string; prompt: string }>;
  };
  /** Optional: send a `skills` allowlist with each request, narrowing which
   *  skills the server splices into the system prompt. */
  skills?: string[];
  /** Wrap-around classes for the outer container. */
  className?: string;
  children?: ReactNode;
}

export function AiUi({
  endpoint = "/api/chat",
  agentName = "AI",
  componentLibrary = openuiLibrary,
  conversationStarters,
  skills,
  className = "h-screen w-screen overflow-hidden relative",
}: AiUiProps) {
  return (
    <div className={className}>
      <FullScreen
        processMessage={async ({ messages, abortController }) => {
          return fetch(endpoint, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              messages: openAIMessageFormat.toApi(messages),
              ...(skills && skills.length > 0 ? { skills } : {}),
            }),
            signal: abortController.signal,
          });
        }}
        streamProtocol={openAIAdapter()}
        componentLibrary={componentLibrary as never}
        agentName={agentName}
        conversationStarters={conversationStarters}
      />
    </div>
  );
}
