const MARKER = "@@QA@@";

export function parseQA(raw) {
  if (!raw.startsWith(MARKER)) return null;
  try {
    const envelope = JSON.parse(raw.slice(MARKER.length));
    return Array.isArray(envelope.questions) && envelope.questions.length > 0
      ? envelope.questions
      : null;
  } catch {
    return null;
  }
}
