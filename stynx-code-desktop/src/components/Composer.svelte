<script>
  import { isStreaming } from "../lib/stores.js";
  import { send } from "../lib/events.js";
  import { cancel } from "../lib/api.js";

  let draft = "";

  function submit() {
    const text = draft;
    draft = "";
    send(text, null);
  }

  function onKeydown(event) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      submit();
    }
  }
</script>

<div class="composer">
  <textarea
    bind:value={draft}
    on:keydown={onKeydown}
    placeholder="Ask stynx…"
    rows="3"
    spellcheck="false"
  ></textarea>
  <div class="actions">
    {#if $isStreaming}
      <button class="stop" on:click={() => cancel()} title="Stop">■</button>
    {:else}
      <button class="go" on:click={submit} disabled={!draft.trim()} title="Send">↑</button>
    {/if}
  </div>
</div>

<style>
  .composer {
    margin: 8px 16px 14px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 10px 12px;
    display: flex;
    align-items: flex-end;
    gap: 10px;
  }

  textarea {
    flex: 1;
    background: none;
    border: none;
    resize: none;
    max-height: 200px;
  }

  .actions button {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    font-size: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .go {
    background: var(--accent);
    color: white;
  }

  .go:disabled {
    background: var(--bg-panel);
    color: var(--text-dim);
  }

  .stop {
    background: var(--danger);
    color: white;
    font-size: 12px;
  }
</style>
