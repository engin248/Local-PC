import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauriRuntime } from "./runtime";

export type LiveFeedEvent = {
  event_type: string;
  source: string;
  message: string;
  task_id?: string | null;
  metadata_json?: string | null;
  timestamp: string;
};

export type LiveFeedHandler = (event: LiveFeedEvent) => void;

const eventTypes = [
  "live-feed",
  "command-submitted",
  "burhan-dispatch",
  "agent-status",
  "report-returned",
  "alarm-code",
  "critical-error",
] as const;

export async function subscribeLiveFeed(handler: LiveFeedHandler): Promise<UnlistenFn[]> {
  if (!isTauriRuntime()) return [];
  const unlisteners: UnlistenFn[] = [];
  for (const eventType of eventTypes) {
    const unlisten = await listen<LiveFeedEvent>(eventType, (payload) => {
      const event = payload.payload;
      handler({
        ...event,
        event_type: event.event_type || eventType,
      });
    });
    unlisteners.push(unlisten);
  }
  return unlisteners;
}

export function parseMetadata<T>(metadata?: string | null): T | null {
  if (!metadata) return null;
  try {
    return JSON.parse(metadata) as T;
  } catch {
    return null;
  }
}
