<script>
  import { info, isStreaming } from "../../lib/stores.js";

  export let onOpenWorkspace;

  let draft = "";

  $: if ($info) draft = $info.workspacePath;
</script>

<section>
  <h3 class="side-title">Workspace</h3>
  <div class="side-row">
    <input bind:value={draft} placeholder="/path/to/project" spellcheck="false" />
    <button
      class="btn-primary"
      on:click={() => onOpenWorkspace(draft.trim() || null, null)}
      disabled={$isStreaming}>Open</button
    >
  </div>
  {#if $info}
    <div class="side-dim">{$info.projectName}</div>
  {/if}
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
</style>
