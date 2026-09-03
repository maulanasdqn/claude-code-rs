import { changes, showFiles } from "./stores.js";
import { computeDiff } from "./diff.js";
import { readFile } from "./api.js";

let changeCounter = 0;

export function recordFileChange(name, inputBuffer) {
  let args;
  try {
    args = JSON.parse(inputBuffer);
  } catch {
    return;
  }
  const path = args.file_path ?? args.path;
  if (!path) return;

  if (name === "file_edit" && typeof args.old_string === "string" && typeof args.new_string === "string") {
    const diff = computeDiff(args.old_string.split("\n"), args.new_string.split("\n"));
    pushChange({
      path,
      kind: name,
      diff,
      lines: null,
      adds: diff.filter((line) => line.kind === "added").length,
      removes: diff.filter((line) => line.kind === "removed").length,
    });
    return;
  }

  if (name === "file_write") {
    readFile(path)
      .then((content) => {
        const lines = content.split("\n");
        pushChange({ path, kind: name, diff: null, lines, adds: lines.length, removes: 0 });
      })
      .catch(() => {
        pushChange({ path, kind: name, diff: null, lines: [], adds: 0, removes: 0 });
      });
  }
}

function pushChange(change) {
  changes.update((list) => [
    ...list,
    { id: `change-${++changeCounter}`, name: change.path.split("/").pop(), ...change },
  ]);
  showFiles.set(true);
}
