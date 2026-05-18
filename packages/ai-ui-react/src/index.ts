export { AiUi } from "./AiUi";
export type { AiUiProps } from "./AiUi";

// Re-export the stock OpenUI library so consumers don't have to depend on
// `@openuidev/react-ui` separately when they're happy with the defaults.
export {
  openuiChatLibrary as defaultLibrary,
  openuiChatPromptOptions as defaultPromptOptions,
} from "@openuidev/react-ui/genui-lib";

// Re-export the open-AI adapter + message-format helpers.
export { openAIAdapter, openAIMessageFormat } from "@openuidev/react-headless";

// The push-channel hook: subscribes to /api/events and applies push events
// to a slot map.
export { useAiUiPush } from "./useAiUiPush";
export type { PushEvent } from "./useAiUiPush";
