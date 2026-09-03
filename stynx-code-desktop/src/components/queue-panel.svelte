<script>
  import { messageQueue } from "../lib/stores.js";
  import { dequeue } from "../lib/messaging.js";
</script>

{#if $messageQueue.length > 0}
  <div class="queue">
    <div class="head">Queue · {$messageQueue.length}</div>
    {#each $messageQueue as queued, index}
      <div class="queued">
        <span>{queued.text || "(image)"}</span>
        <button on:click={() => dequeue(index)}>✕</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .queue {
    position: absolute;
    right: 16px;
    bottom: 120px;
    width: 260px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  }

  .head {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-dim);
  }

  .queued {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    font-size: 12px;
    background: var(--bg-panel);
    border-radius: 8px;
    padding: 6px 8px;
  }

  .queued span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .queued button {
    color: var(--text-dim);
  }
</style>
