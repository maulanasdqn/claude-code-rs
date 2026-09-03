<script>
  import { parseMarkdown } from "../lib/markdown.js";

  export let raw;

  $: blocks = parseMarkdown(raw);
</script>

<div class="markdown">
  {#each blocks as block}
    {#if block.kind === "code"}
      <pre>{block.text}</pre>
    {:else if block.kind === "heading"}
      <div class="heading" class:big={block.level <= 2}>{@html block.html}</div>
    {:else if block.kind === "bullet"}
      <div class="listrow"><span class="marker">•</span><span class="body">{@html block.html}</span></div>
    {:else if block.kind === "numbered"}
      <div class="listrow"><span class="marker">{block.number}.</span><span class="body">{@html block.html}</span></div>
    {:else if block.kind === "rule"}
      <hr />
    {:else if block.kind === "table"}
      <div class="table-wrap">
        <table>
          <thead>
            <tr>{#each block.header as cell}<th>{@html cell}</th>{/each}</tr>
          </thead>
          <tbody>
            {#each block.rows as row}
              <tr>{#each row as cell}<td>{@html cell}</td>{/each}</tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else if block.kind === "blank"}
      <div class="blank"></div>
    {:else}
      <div class="text">{@html block.html}</div>
    {/if}
  {/each}
</div>

<style>
  .markdown {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 14px;
    line-height: 1.55;
    min-width: 0;
  }

  .text,
  .body {
    white-space: pre-wrap;
    word-break: break-word;
  }

  .heading {
    font-weight: 600;
    margin-top: 6px;
  }

  .heading.big {
    font-size: 16px;
  }

  .listrow {
    display: flex;
    gap: 8px;
  }

  .marker {
    color: var(--text-dim);
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
  }

  .body {
    flex: 1;
    min-width: 0;
  }

  hr {
    border: none;
    border-top: 1px solid var(--border);
    margin: 8px 0;
  }

  .blank {
    height: 6px;
  }

  .markdown :global(code) {
    background: var(--bg-card);
    border-radius: 4px;
    padding: 1px 5px;
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-size: 12.5px;
  }

  .markdown :global(a) {
    color: var(--accent);
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
    margin: 4px 0;
  }

  .table-wrap {
    overflow-x: auto;
    margin: 4px 0;
  }

  table {
    border-collapse: collapse;
    background: var(--bg-card);
    border-radius: 8px;
    font-size: 13px;
  }

  th,
  td {
    text-align: left;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border);
  }

  th {
    font-weight: 600;
  }

  tr:last-child td {
    border-bottom: none;
  }
</style>
