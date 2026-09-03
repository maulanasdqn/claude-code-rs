import { get, writable } from "svelte/store";
import { listProjectFiles } from "./api.js";

export const fileIndex = writable([]);

export async function rebuildFileIndex(root) {
  if (!root) {
    fileIndex.set([]);
    return;
  }
  try {
    fileIndex.set(await listProjectFiles(root));
  } catch {
    fileIndex.set([]);
  }
}

export function currentMention(text) {
  const at = text.lastIndexOf("@");
  if (at < 0) return null;
  const after = text.slice(at + 1);
  if (after.includes(" ") || after.includes("\n")) return null;
  if (at > 0 && !/\s/.test(text[at - 1])) return null;
  return after;
}

export function mentionSuggestions(query) {
  const index = get(fileIndex);
  const matches = query
    ? index.filter((path) => path.toLowerCase().includes(query.toLowerCase()))
    : index;
  return matches.slice(0, 8);
}

export function applyMention(text, path) {
  const at = text.lastIndexOf("@");
  if (at < 0) return text;
  return `${text.slice(0, at)}@${path} `;
}
