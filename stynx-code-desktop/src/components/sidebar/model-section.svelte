<script>
  import { info, status, tokens, isStreaming } from "../../lib/stores.js";
  import { applyModel } from "../../lib/session-actions.js";

  export let onOpenWorkspace;

  let modelDraft = "";

  $: if ($info) modelDraft = $info.modelId;

  function switchProvider(event) {
    onOpenWorkspace($info?.workspacePath ?? null, event.target.value);
  }
</script>

<section>
  <h3 class="side-title">Model</h3>
  {#if $info}
    <label class="field">
      <span>Main agent</span>
      <select value={$info.currentProvider} on:change={switchProvider} disabled={$isStreaming}>
        {#each $info.mainProviders as provider}
          <option value={provider}>{provider}</option>
        {/each}
      </select>
    </label>
    {#if $info.currentProvider === "claude"}
      <label class="field">
        <span>Claude model</span>
        <select bind:value={modelDraft} on:change={() => applyModel(modelDraft)}>
          {#each $info.claudeModels as id}
            <option value={id}>{id}</option>
          {/each}
        </select>
      </label>
    {:else}
      <label class="field">
        <span>Model id</span>
        <input bind:value={modelDraft} on:change={() => applyModel(modelDraft)} spellcheck="false" />
      </label>
    {/if}
  {/if}
  <div class="side-dim">Status: {$status}</div>
  <div class="side-dim">Tokens: in {$tokens.input} · out {$tokens.output}</div>
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    font-size: 13px;
  }

  .field select,
  .field input {
    max-width: 150px;
  }
</style>
