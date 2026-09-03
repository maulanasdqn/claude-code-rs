const TOOL_TITLE_CHARS = 80;

export function titleFor(name, input) {
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

export function toolInputField(json, keys) {
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

function truncate(value, max) {
  if (!value) return null;
  return value.length > max ? value.slice(0, max) : value;
}
