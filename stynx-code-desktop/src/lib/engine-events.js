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
  nextId,
} from "./stores.js";
import { parseQA } from "./qa.js";
import { listSessions, respondWorkspaceMessage } from "./api.js";
import { drainQueue } from "./messaging.js";
import { recordFileChange } from "./changes.js";
import { titleFor, toolInputField } from "./tool-title.js";

const TOOL_DETAIL_CHARS = 2000;

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
      startTool(event.name, event.id);
      break;
    case "toolInput":
      accumulateToolInput(event.jsonChunk);
      break;
    case "toolOutput":
      updateTool(event.name, (tool) => {
        tool.detail = (tool.detail + event.chunk).slice(-TOOL_DETAIL_CHARS);
      });
      break;
    case "toolResult": {
      let completedToolId = null;
      updateTool(event.name, (tool) => {
        tool.running = false;
        tool.isError = event.isError;
        if (!tool.detail) tool.detail = event.output.slice(0, TOOL_DETAIL_CHARS);
        if (tool.name === "file_write") tool.badge = "A";
        if (tool.name === "file_edit") tool.badge = "M";
        completedToolId = tool.toolId;
      });
      if (!event.isError && ["file_write", "file_edit"].includes(event.name) && completedToolId) {
        recordFileChange(event.name, toolInputBuffers.get(completedToolId) ?? "");
      }
      currentStreamKind = null;
      break;
    }
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
  drainQueue();
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

function startTool(name, id) {
  currentStreamKind = null;
  currentToolId = id;
  toolInputBuffers.set(id, "");
  feed.update((items) => [
    ...items,
    {
      id: nextId(),
      role: "tool",
      tool: {
        toolId: id,
        name,
        title: name,
        subtitle: null,
        detail: "",
        running: true,
        isError: false,
        badge: null,
      },
    },
  ]);
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
