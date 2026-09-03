<script>
  import { info, isStreaming } from "../../lib/stores.js";
  import { recentWorkspaces, forgetWorkspace } from "../../lib/workspaces.js";

  export let onOpenWorkspace;

  let draft = "";
  let recents = recentWorkspaces();

  $: if ($info) {
    draft = $info.workspacePath;
    recents = recentWorkspaces();
  }

  function shortName(path) {
    return path.split("/").filter(Boolean).pop() ?? path;
  }

  function forget(path) {
    recents = forgetWorkspace(path);
  }
</script>

<section>
  <h3 class="side-title">Workspace</h3>
  {#each recents as path (path)}
    <div class="recent">
      <button
        class="open"
        class:current={path === $info?.workspacePath}
        title={path}
        on:click={() => onOpenWorkspace(path, null)}
        disabled={$isStreaming}
      >
        <span class="folder">{path === $info?.workspacePath ? "📂" : "📁"}</span>
        <span class="name">{shortName(path)}</span>
      </button>
      <button class="forget" title="Remove from list" on:click={() => forget(path)}>✕</button>
    </div>
  {/each}
  <div class="side-row">
    <input bind:value={draft} placeholder="/path/to/project" spellcheck="false" />
    <button
      class="btn-primary"
      on:click={() => onOpenWorkspace(draft.trim() || null, null)}
      disabled={$isStreaming}>Open</button
    >
  </div>
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .recent {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .open {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 8px;
    border-radius: 8px;
    font-size: 13px;
  }

  .open:hover {
    background: var(--accent-soft);
  }

  .open.current .name {
    font-weight: 600;
  }

  .folder {
    font-size: 12px;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .forget {
    color: var(--text-dim);
    font-size: 11px;
    padding: 4px;
  }

  .forget:hover {
    color: var(--danger);
  }
</style>
