import { get } from "svelte/store";
import {
  info,
  status,
  sessions,
  interns,
  feed,
  isStreaming,
  mode,
  thinking,
  resetTranscript,
  nextId,
} from "./stores.js";
import {
  initSession,
  listSessions,
  loadSession,
  newSession,
  deleteSession,
  setMode,
  setThinking,
  setModel,
  setProviderKey,
} from "./api.js";
import { rememberWorkspace } from "./workspaces.js";
import { rebuildFileIndex } from "./mentions.js";

export async function openWorkspace(path, provider) {
  status.set("Starting…");
  const result = await initSession(path, provider);
  const previous = get(info);
  if (previous && previous.workspacePath !== result.workspacePath) {
    resetTranscript();
  }
  info.set(result);
  interns.set(result.interns);
  rememberWorkspace(result.workspacePath);
  rebuildFileIndex(result.workspacePath);
  status.set("Ready");
  sessions.set(await listSessions());
}

export async function switchProvider(provider) {
  const current = get(info);
  await openWorkspace(current?.workspacePath ?? null, provider);
}

export async function openSession(id) {
  if (get(isStreaming)) return;
  const turns = await loadSession(id);
  resetTranscript();
  feed.set(turns.map((turn) => ({ id: nextId(), role: turn.role, text: turn.text })));
  status.set("Loaded session");
}

export async function startNewSession() {
  if (get(isStreaming)) return;
  await newSession();
  resetTranscript();
  status.set("Ready");
}

export async function removeSession(id) {
  await deleteSession(id);
  sessions.set(await listSessions());
}

export async function applyMode(value) {
  mode.set(value);
  await setMode(value);
}

export async function applyThinking(enabled) {
  thinking.set(enabled);
  await setThinking(enabled);
}

export async function applyModel(model) {
  const id = await setModel(model.trim());
  const current = get(info);
  if (current) info.set({ ...current, modelId: id });
  return id;
}

export async function saveInternKey(envName, value) {
  const trimmed = value.trim();
  if (!trimmed) return;
  interns.set(await setProviderKey(envName, trimmed));
}
