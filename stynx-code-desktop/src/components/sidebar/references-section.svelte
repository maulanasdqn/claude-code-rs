<script>
  import { references } from "../../lib/stores.js";
  import { addReferenceFiles, addReferenceFromUrl, removeReference } from "../../lib/references.js";

  let fileInput;
  let urlDraft = "";

  function onPick(event) {
    addReferenceFiles([...event.target.files]);
    event.target.value = "";
  }

  async function addUrl() {
    if (await addReferenceFromUrl(urlDraft)) urlDraft = "";
  }
</script>

<section>
  <h3 class="side-title">References</h3>
  <button class="btn-ghost" on:click={() => fileInput.click()}>＋ Add reference</button>
  <input type="file" multiple bind:this={fileInput} on:change={onPick} hidden />
  <div class="side-row">
    <input bind:value={urlDraft} placeholder="https://…" spellcheck="false" />
    <button class="btn-primary" on:click={addUrl} disabled={!urlDraft.trim()}>Fetch</button>
  </div>
  {#each $references as doc (doc.id)}
    <div class="ref">
      <span class="icon">📄</span>
      <span class="meta">
        <span class="name">{doc.name}</span>
        <span class="side-dim">{doc.text ? "reference" : "no text extracted"}</span>
      </span>
      <button class="remove" on:click={() => removeReference(doc.id)}>✕</button>
    </div>
  {:else}
    <div class="side-dim">No reference documents</div>
  {/each}
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .ref {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 2px;
  }

  .icon {
    font-size: 12px;
  }

  .meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .name {
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .remove {
    color: var(--text-dim);
    font-size: 11px;
  }

  .remove:hover {
    color: var(--danger);
  }
</style>
