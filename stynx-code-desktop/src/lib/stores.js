import { writable } from "svelte/store";

export const feed = writable([]);
export const status = writable("Starting…");
export const isStreaming = writable(false);
export const info = writable(null);
export const sessions = writable([]);
export const interns = writable([]);
export const permissionPrompt = writable(null);
export const question = writable(null);
export const tokens = writable({ input: 0, output: 0 });
export const mode = writable("normal");
export const thinking = writable(true);
export const messageQueue = writable([]);

let counter = 0;
export const nextId = () => `item-${++counter}`;

export function resetTranscript() {
  feed.set([]);
  tokens.set({ input: 0, output: 0 });
  permissionPrompt.set(null);
  question.set(null);
}
