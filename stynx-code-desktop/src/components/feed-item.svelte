<script>
  import Markdown from "./markdown.svelte";
  import ToolCard from "./tool-card.svelte";
  import { dataUrl } from "../lib/images.js";

  export let item;
</script>

{#if item.role === "user"}
  <div class="user-row">
    <div class="user-stack">
      {#if item.images?.length}
        <div class="user-images">
          {#each item.images as image}
            <img src={dataUrl(image)} alt="attachment" />
          {/each}
        </div>
      {/if}
      {#if item.text}
        <div class="user-bubble">{item.text}</div>
      {/if}
      {#if item.referenceCount}
        <div class="ref-note">📄 {item.referenceCount} reference{item.referenceCount === 1 ? "" : "s"}</div>
      {/if}
    </div>
  </div>
{:else if item.role === "assistant"}
  <div class="assistant">
    <div class="label">Stynx</div>
    <Markdown raw={item.text} />
  </div>
{:else if item.role === "thinking"}
  <details class="thinking">
    <summary>Thinking…</summary>
    <div class="thinking-body">{item.text}</div>
  </details>
{:else if item.role === "tool"}
  <ToolCard tool={item.tool} />
{:else if item.role === "compact"}
  <div class="compact">
    <hr />
    <span>Context compacted · {item.originalTurns} turns summarised</span>
    <hr />
  </div>
{/if}

<style>
  .user-row {
    display: flex;
    justify-content: flex-end;
  }

  .user-stack {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 6px;
    max-width: 70%;
  }

  .user-images {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .user-images img {
    width: 120px;
    height: 120px;
    object-fit: cover;
    border-radius: 12px;
  }

  .ref-note {
    font-size: 11px;
    color: var(--text-dim);
  }

  .user-bubble {
    background: var(--accent);
    color: white;
    padding: 9px 14px;
    border-radius: 16px;
    font-size: 14px;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .assistant {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-width: 100%;
  }

  .label {
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-dim);
  }

  .thinking {
    font-size: 13px;
    color: var(--text-dim);
  }

  .thinking summary {
    cursor: pointer;
    font-size: 12px;
  }

  .thinking-body {
    white-space: pre-wrap;
    padding: 6px 0 0 12px;
    border-left: 2px solid var(--border);
    margin-top: 4px;
  }

  .compact {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--text-dim);
    font-size: 11px;
  }

  .compact hr {
    flex: 1;
    border: none;
    border-top: 1px solid var(--border);
  }
</style>
