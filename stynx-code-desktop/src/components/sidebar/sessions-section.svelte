<script>
  import { sessions, isStreaming } from "../../lib/stores.js";
  import { openSession, startNewSession, removeSession } from "../../lib/session-actions.js";
</script>

<section>
  <h3 class="side-title">Sessions</h3>
  <button class="btn-ghost" on:click={startNewSession} disabled={$isStreaming}>
    ＋ New conversation
  </button>
  {#each $sessions as summary (summary.id)}
    <div class="row">
      <button class="session" on:click={() => openSession(summary.id)}>
        <span class="title">{summary.title || "Untitled"}</span>
        <span class="side-dim">{summary.messageCount} messages</span>
      </button>
      <button class="delete" title="Delete" on:click={() => removeSession(summary.id)}>✕</button>
    </div>
  {:else}
    <div class="side-dim">No saved sessions</div>
  {/each}
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .session {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 5px 8px;
    border-radius: 8px;
  }

  .session:hover {
    background: var(--accent-soft);
  }

  .title {
    font-size: 13px;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .delete {
    color: var(--text-dim);
    padding: 4px;
    font-size: 11px;
  }

  .delete:hover {
    color: var(--danger);
  }
</style>
