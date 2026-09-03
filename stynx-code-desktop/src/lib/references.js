import { get } from "svelte/store";
import { references, status } from "./stores.js";
import { fetchReference } from "./api.js";

const MAX_CHARS = 60_000;
let counter = 0;
let dirty = false;

export function addReferenceFiles(files) {
  for (const file of files) {
    const reader = new FileReader();
    reader.onload = () => {
      const text = typeof reader.result === "string" ? reader.result.slice(0, MAX_CHARS) : "";
      push(file.name, file.name, looksBinary(text) ? "" : text);
    };
    reader.onerror = () => push(file.name, file.name, "");
    reader.readAsText(file);
  }
}

export async function addReferenceFromUrl(link) {
  const trimmed = link.trim();
  if (!/^https?:\/\/\S+$/.test(trimmed)) return false;
  const name = trimmed.split("/").filter(Boolean).pop() ?? trimmed;
  status.set(`Fetching ${name}…`);
  try {
    const result = await fetchReference(trimmed);
    const text = result.contentType.includes("html") ? stripHtml(result.body) : result.body;
    push(name, trimmed, text.slice(0, MAX_CHARS));
  } catch {
    status.set("Fetch failed");
    return true;
  }
  if (get(status).startsWith("Fetching")) status.set("Ready");
  return true;
}

export function removeReference(id) {
  references.update((docs) => docs.filter((doc) => doc.id !== id));
  dirty = true;
}

/// Wraps not-yet-sent reference documents around the outgoing message text.
export function injectReferences(text) {
  const docs = get(references);
  if (docs.length === 0 || !dirty) return { text, count: 0 };
  dirty = false;
  const blocks = docs
    .map((doc) => `## ${doc.name}\n${doc.text || `(no extractable text — ${doc.path})`}`)
    .join("\n\n");
  const intro =
    "The user attached the following reference document(s). Treat their content as authoritative context for this and following requests.";
  return {
    text: `${intro}\n\n<reference_documents>\n${blocks}\n</reference_documents>\n\n${text}`,
    count: docs.length,
  };
}

function push(name, path, text) {
  references.update((docs) => [...docs, { id: `ref-${++counter}`, name, path, text }]);
  dirty = true;
}

function looksBinary(text) {
  return text.includes("\uFFFD");
}

function stripHtml(html) {
  let s = html;
  s = s.replace(/<script[^>]*>[\s\S]*?<\/script>/gi, " ");
  s = s.replace(/<style[^>]*>[\s\S]*?<\/style>/gi, " ");
  s = s.replace(/<[^>]+>/g, " ");
  s = s
    .replaceAll("&nbsp;", " ")
    .replaceAll("&amp;", "&")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&#39;", "'")
    .replaceAll("&quot;", '"');
  s = s.replace(/[ \t]+/g, " ");
  s = s.replace(/\n{3,}/g, "\n\n");
  return s.trim();
}
