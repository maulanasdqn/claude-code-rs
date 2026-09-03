let lineCounter = 0;

export function computeDiff(oldLines, newLines) {
  const n = oldLines.length;
  const m = newLines.length;
  const lcs = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      lcs[i][j] =
        oldLines[i] === newLines[j]
          ? lcs[i + 1][j + 1] + 1
          : Math.max(lcs[i + 1][j], lcs[i][j + 1]);
    }
  }

  const result = [];
  let i = 0;
  let j = 0;
  let oldNum = 1;
  let newNum = 1;
  const push = (kind, text, oldNumber, newNumber) =>
    result.push({ id: `line-${++lineCounter}`, kind, text, oldNumber, newNumber });

  while (i < n && j < m) {
    if (oldLines[i] === newLines[j]) {
      push("context", oldLines[i], oldNum++, newNum++);
      i++;
      j++;
    } else if (lcs[i + 1][j] >= lcs[i][j + 1]) {
      push("removed", oldLines[i], oldNum++, null);
      i++;
    } else {
      push("added", newLines[j], null, newNum++);
      j++;
    }
  }
  while (i < n) {
    push("removed", oldLines[i], oldNum++, null);
    i++;
  }
  while (j < m) {
    push("added", newLines[j], null, newNum++);
    j++;
  }
  return result;
}

export function diffQuote(filePath, lines) {
  const name = filePath.split("/").pop();
  const body = lines
    .map((line) => {
      if (line.kind === "added") return `+${line.text}`;
      if (line.kind === "removed") return `-${line.text}`;
      return ` ${line.text}`;
    })
    .join("\n");
  return `[diff: ${name}]\n\`\`\`diff\n${body}\n\`\`\`\n`;
}
