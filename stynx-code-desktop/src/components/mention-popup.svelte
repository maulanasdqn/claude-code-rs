<script>
  import { composerDraft } from "../lib/stores.js";
  import { currentMention, mentionSuggestions, applyMention } from "../lib/mentions.js";

  $: query = currentMention($composerDraft);
  $: suggestions = query === null ? [] : mentionSuggestions(query);

  function pick(path) {
    $composerDraft = applyMention($composerDraft, path);
  }
</script>

{#if suggestions.length > 0}
  <div class="popup">
    {#each suggestions as path (path)}
      <button on:click={() => pick(path)}>
        <span class="icon">📄</span>
        <span class="path">{path}</span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .popup {
    display: flex;
    flex-direction: column;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 4px;
    max-height: 240px;
    overflow-y: auto;
  }

  button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-radius: 8px;
    text-align: left;
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-size: 12px;
  }

  button:hover {
    background: var(--accent-soft);
  }

  .icon {
    font-size: 11px;
  }

  .path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
