import { listen } from "@tauri-apps/api/event";
import { get } from "svelte/store";
import {
  feed,
  status,
  isStreaming,
  sessions,
  permissionPrompt,
  question,
  tokens,
  messageQueue,
  nextId,
} from "./stores.js";
import { parseQA } from "./qa.js";
import { listSessions, sendMessage, respondWorkspaceMessage } from "./api.js";

const TOOL_DETAIL_CHARS = 2000;
const TOOL_TITLE_CHARS = 80;

let currentStreamKind = null;
let currentToolId = "";
const toolInputBuffers = new Map();

export function attachEngineEvents() {
  return listen("engine-event", (event) => reduce(event.payload));
}

function reduce(event) {
  if (event.type !== "retryNotice" && get(status).startsWith("Overloaded")) {
    status.set("Thinking…");
  }
  switch (event.type) {
    case "textDelta":
      appendDelta("assistant", event.text);
      break;
    case "thinkingDelta":
      appendDelta("thinking", event.text);
      break;
    case "toolStart":
      currentStreamKind = null;
      currentToolId = event.id;
      toolInputBuffers.set(event.id, "");
      feed.update((items) => [
        ...items,
        {
          id: nextId(),
          role: "tool",
          tool: {
            toolId: event.id,
            name: event.name,
            title: event.name,
            subtitle: null,
            detail: "",
            running: true,
            isError: false,
            badge: null,
          },
        },
      ]);
      break;
    case "toolInput":
      accumulateToolInput(event.jsonChunk);
      break;
    case "toolOutput":
      updateTool(event.name, (tool) => {
        tool.detail = (tool.detail + event.chunk).slice(-TOOL_DETAIL_CHARS);
      });
      break;
    case "toolResult":
      updateTool(event.name, (tool) => {
        tool.running = false;
        tool.isError = event.isError;
        if (!tool.detail) tool.detail = event.output.slice(0, TOOL_DETAIL_CHARS);
        if (tool.name === "file_write") tool.badge = "A";
        if (tool.name === "file_edit") tool.badge = "M";
      });
      currentStreamKind = null;
      break;
    case "usage":
      tokens.set({ input: event.inputTokens, output: event.outputTokens });
      break;
    case "permissionRequest":
      permissionPrompt.set({
        id: event.id,
        toolName: event.toolName,
        description: event.description,
      });
      break;
    case "askUserRequest":
      question.set({ id: event.id, question: event.question, qa: parseQA(event.question) });
      break;
    case "workspaceMessageRequest":
      respondWorkspaceMessage(
        event.id,
        "(workspace routing is not available in the desktop scaffold yet)",
      ).catch(() => {});
      break;
    case "retryNotice":
      status.set(
        `Overloaded — retry ${event.attempt}/${event.maxAttempts} in ${Math.round(event.delayMs / 1000)}s`,
      );
      break;
    case "error":
      currentStreamKind = null;
      feed.update((items) => [
        ...items,
        { id: nextId(), role: "assistant", text: `⚠️ ${event.message}` },
      ]);
      break;
    case "compacted":
      currentStreamKind = null;
      feed.update((items) => [
        ...items,
        { id: nextId(), role: "compact", originalTurns: event.originalTurns },
      ]);
      break;
    case "idle":
      handleIdle();
      break;
    default:
      break;
  }
}

function handleIdle() {
  isStreaming.set(false);
  currentStreamKind = null;
  if (get(status) === "Thinking…") status.set("Ready");
  listSessions()
    .then((list) => sessions.set(list))
    .catch(() => {});
  const queue = get(messageQueue);
  if (queue.length > 0) {
    const [next, ...rest] = queue;
    messageQueue.set(rest);
    dispatch(next.text, next.images);
  }
}

export function dispatch(text, images) {
  feed.update((items) => [
    ...items,
    { id: nextId(), role: "user", text, images: images ?? [] },
  ]);
  currentStreamKind = null;
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

export function send(text, images) {
  const trimmed = text.trim();
  if (!trimmed && (!images || images.length === 0)) return;
  if (get(isStreaming)) {
    messageQueue.update((queue) => [...queue, { text: trimmed, images }]);
    return;
  }
  dispatch(trimmed, images);
}

function appendDelta(role, delta) {
  feed.update((items) => {
    const last = items[items.length - 1];
    if (currentStreamKind === role && last && last.role === role) {
      const merged = [...items];
      merged[merged.length - 1] = { ...last, text: last.text + delta };
      return merged;
    }
    currentStreamKind = role;
    return [...items, { id: nextId(), role, text: delta }];
  });
}

function accumulateToolInput(chunk) {
  if (!currentToolId) return;
  const buffer = (toolInputBuffers.get(currentToolId) ?? "") + chunk;
  toolInputBuffers.set(currentToolId, buffer);
  const toolId = currentToolId;
  feed.update((items) => {
    const index = items.findLastIndex((item) => item.tool?.toolId === toolId);
    if (index < 0) return items;
    const merged = [...items];
    const tool = { ...merged[index].tool };
    if (tool.name === "message_workspace") {
      const target = toolInputField(buffer, ["target"]);
      const task = toolInputField(buffer, ["task"]);
      if (target) tool.title = target;
      if (task) tool.subtitle = task;
    } else {
      const title = titleFor(tool.name, buffer);
      if (title) tool.title = title;
    }
    merged[index] = { ...merged[index], tool };
    return merged;
  });
}

function updateTool(name, mutate) {
  feed.update((items) => {
    let index = items.findLastIndex((item) => item.tool?.running && item.tool?.name === name);
    if (index < 0) index = items.findLastIndex((item) => item.tool?.name === name);
    if (index < 0) return items;
    const merged = [...items];
    const tool = { ...merged[index].tool };
    mutate(tool);
    merged[index] = { ...merged[index], tool };
    return merged;
  });
}

function titleFor(name, input) {
  switch (name) {
    case "bash":
      return truncate(toolInputField(input, ["command"]), TOOL_TITLE_CHARS);
    case "file_write":
    case "file_edit":
    case "read": {
      const path = toolInputField(input, ["path", "file_path"]);
      return path ? path.split("/").pop() : null;
    }
    case "glob":
    case "grep":
      return toolInputField(input, ["pattern", "query"]);
    case "web_fetch":
    case "web_search":
      return toolInputField(input, ["url", "query"]);
    default:
      if (name.startsWith("delegate_to_")) {
        return truncate(toolInputField(input, ["task"]), TOOL_TITLE_CHARS);
      }
      return null;
  }
}

function truncate(value, max) {
  if (!value) return null;
  return value.length > max ? value.slice(0, max) : value;
}

function toolInputField(json, keys) {
  for (const key of keys) {
    const marker = `"${key}"`;
    const at = json.indexOf(marker);
    if (at < 0) continue;
    const colon = json.indexOf(":", at + marker.length);
    if (colon < 0) continue;
    const open = json.indexOf('"', colon + 1);
    if (open < 0) continue;
    const close = json.indexOf('"', open + 1);
    if (close < 0) continue;
    return json.slice(open + 1, close);
  }
  return null;
}
