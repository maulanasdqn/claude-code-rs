<script>
  import { parseMarkdown } from "../lib/markdown.js";

  export let raw;

  $: blocks = parseMarkdown(raw);
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
