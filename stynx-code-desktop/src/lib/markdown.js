export function parseMarkdown(text) {
  const out = [];
  const fence = /```[^\n`]*\n?([\s\S]*?)```/g;
  let last = 0;
  let match;
  while ((match = fence.exec(text)) !== null) {
    if (match.index > last) {
      out.push({ kind: "text", html: inline(text.slice(last, match.index)) });
    }
    out.push({ kind: "code", text: match[1] });
    last = match.index + match[0].length;
  }
  if (last < text.length) {
    out.push({ kind: "text", html: inline(text.slice(last)) });
  }
  return out;
}

function escapeHtml(s) {
  return s
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function inline(text) {
  let s = escapeHtml(text);
  s = s.replace(/`([^`\n]+)`/g, "<code>$1</code>");
  s = s.replace(/\*\*([^*\n]+)\*\*/g, "<strong>$1</strong>");
  s = s.replace(/^###+\s+(.+)$/gm, "<strong>$1</strong>");
  s = s.replace(/^##\s+(.+)$/gm, "<strong class='h'>$1</strong>");
  s = s.replace(/^#\s+(.+)$/gm, "<strong class='h'>$1</strong>");
  s = s.replace(/^[-*]\s+/gm, "•&nbsp;");
  return s;
}
