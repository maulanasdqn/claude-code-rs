<script>
  import { interns } from "../../lib/stores.js";
  import { saveInternKey } from "../../lib/session-actions.js";

  let keyEntryFor = null;
  let keyDraft = "";

  async function save(intern) {
    const value = keyDraft;
    keyEntryFor = null;
    keyDraft = "";
    await saveInternKey(intern.keyEnv, value);
  }
</script>

<section>
  <h3 class="side-title">Interns</h3>
  {#each $interns as intern (intern.name)}
    <div class="intern">
      <div class="head">
        <span class="side-mono">{intern.name}</span>
        {#if intern.available}
          <span class="ok">ready</span>
        {:else}
          <button class="add-key" on:click={() => (keyEntryFor = intern.name)}>add key</button>
        {/if}
      </div>
      <div class="side-dim">{intern.provider} · {intern.model}</div>
      {#if keyEntryFor === intern.name}
        <div class="side-row">
          <input
            bind:value={keyDraft}
            placeholder={intern.keyEnv}
            type="password"
            spellcheck="false"
          />
          <button class="btn-primary" on:click={() => save(intern)}>Save</button>
        </div>
      {/if}
    </div>
  {:else}
    <div class="side-dim">No interns configured</div>
  {/each}
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .intern {
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .ok {
    color: var(--ok);
    font-size: 11px;
  }

  .add-key {
    font-size: 11px;
    color: var(--accent);
    padding: 2px 6px;
  }
</style>
