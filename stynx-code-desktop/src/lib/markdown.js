export function parseMarkdown(raw) {
  const lines = raw.split("\n");
  const blocks = [];
  let index = 0;

  while (index < lines.length) {
    const line = lines[index];
    const trimmed = line.trim();

    if (trimmed.startsWith("```")) {
      const code = [];
      index++;
      while (index < lines.length && !lines[index].trim().startsWith("```")) {
        code.push(lines[index]);
        index++;
      }
      index++;
      blocks.push({ kind: "code", text: code.join("\n") });
      continue;
    }

    if (isRule(trimmed)) {
      blocks.push({ kind: "rule" });
      index++;
      continue;
    }

    if (isTableRow(trimmed) && index + 1 < lines.length && isTableSeparator(lines[index + 1].trim())) {
      const header = parseTableRow(trimmed);
      const rows = [];
      index += 2;
      while (index < lines.length && isTableRow(lines[index].trim())) {
        rows.push(parseTableRow(lines[index].trim()));
        index++;
      }
      blocks.push({ kind: "table", header: header.map(inline), rows: rows.map((r) => r.map(inline)) });
      continue;
    }

    if (!trimmed) {
      blocks.push({ kind: "blank" });
    } else {
      const heading = headingOf(trimmed);
      const bullet = bulletOf(trimmed);
      const numbered = numberedOf(trimmed);
      if (heading) {
        blocks.push({ kind: "heading", level: heading.level, html: inline(heading.text) });
      } else if (bullet !== null) {
        blocks.push({ kind: "bullet", html: inline(bullet) });
      } else if (numbered) {
        blocks.push({ kind: "numbered", number: numbered.number, html: inline(numbered.text) });
      } else {
        blocks.push({ kind: "paragraph", html: inline(line) });
      }
    }
    index++;
  }
  return blocks;
}

function isRule(line) {
  if (line.length < 3) return false;
  return ["-", "*", "_"].some((marker) => [...line].every((ch) => ch === marker));
}

function isTableRow(line) {
  return line.includes("|") && line.length > 0;
}

function isTableSeparator(line) {
  if (!line.includes("-")) return false;
  const cells = parseTableRow(line);
  return cells.length > 0 && cells.every((cell) => cell && [...cell].every((ch) => "-: ".includes(ch)));
}

function parseTableRow(line) {
  let content = line;
  if (content.startsWith("|")) content = content.slice(1);
  if (content.endsWith("|")) content = content.slice(0, -1);
  return content.split("|").map((cell) => cell.trim());
}

function headingOf(line) {
  const match = /^(#{1,6}) (.+)$/.exec(line);
  return match ? { level: match[1].length, text: match[2] } : null;
}

function bulletOf(line) {
  if (line.startsWith("- ") || line.startsWith("* ")) return line.slice(2);
  return null;
}

function numberedOf(line) {
  const match = /^(\d+)\. (.+)$/.exec(line);
  return match ? { number: match[1], text: match[2] } : null;
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
  s = s.replace(/(^|[^*])\*([^*\n]+)\*(?!\*)/g, "$1<em>$2</em>");
  s = s.replace(/\[([^\]\n]+)\]\((https?:[^)\s]+)\)/g, '<a href="$2" target="_blank" rel="noreferrer">$1</a>');
  return s;
}
