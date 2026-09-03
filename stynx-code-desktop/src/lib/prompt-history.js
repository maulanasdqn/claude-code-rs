const history = [];
let cursor = null;

export function recordPrompt(text) {
  if (history[history.length - 1] !== text) history.push(text);
  cursor = null;
}

export function historyUp(currentInput) {
  if (history.length === 0) return null;
  if (cursor === null) {
    if (currentInput !== "") return null;
    cursor = history.length - 1;
  } else {
    cursor = Math.max(0, cursor - 1);
  }
  return history[cursor];
}

export function historyDown() {
  if (cursor === null) return null;
  const next = cursor + 1;
  if (next >= history.length) {
    cursor = null;
    return "";
  }
  cursor = next;
  return history[next];
}
