<script>
  export let raw;

  $: blocks = parse(raw);

  function parse(text) {
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
</script>

<div class="markdown">
  {#each blocks as block}
    {#if block.kind === "code"}
      <pre>{block.text}</pre>
    {:else}
      <div class="text">{@html block.html}</div>
    {/if}
  {/each}
</div>

<style>
  .markdown {
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 14px;
    line-height: 1.55;
  }

  .text {
    white-space: pre-wrap;
    word-break: break-word;
  }

  .text :global(code) {
    background: var(--bg-card);
    border-radius: 4px;
    padding: 1px 5px;
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-size: 12.5px;
  }

  .text :global(strong.h) {
    font-size: 15px;
  }

  pre {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 10px 12px;
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-size: 12.5px;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
