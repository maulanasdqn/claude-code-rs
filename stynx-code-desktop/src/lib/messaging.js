import { get } from "svelte/store";
import { feed, status, isStreaming, messageQueue, nextId } from "./stores.js";
import { sendMessage } from "./api.js";

export function send(text, images) {
  const trimmed = text.trim();
  if (!trimmed && (!images || images.length === 0)) return;
  if (get(isStreaming)) {
    messageQueue.update((queue) => [...queue, { text: trimmed, images }]);
    return;
  }
  dispatch(trimmed, images);
}

export function dispatch(text, images) {
  feed.update((items) => [
    ...items,
    { id: nextId(), role: "user", text, images: images ?? [] },
  ]);
  isStreaming.set(true);
  status.set("Thinking…");
  sendMessage(text, images && images.length > 0 ? images : null).catch((error) => {
    isStreaming.set(false);
    status.set("Ready");
    feed.update((items) => [
      ...items,
      { id: nextId(), role: "assistant", text: `⚠️ ${error}` },
    ]);
  });
}

export function dequeue(index) {
  messageQueue.update((queue) => queue.filter((_, i) => i !== index));
}

export function drainQueue() {
  const queue = get(messageQueue);
  if (queue.length === 0) return;
  const [next, ...rest] = queue;
  messageQueue.set(rest);
  dispatch(next.text, next.images);
}
