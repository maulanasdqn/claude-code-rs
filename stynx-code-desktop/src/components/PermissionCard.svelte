<script>
  import { respondPermission } from "../lib/api.js";
  import { permissionPrompt } from "../lib/stores.js";

  export let prompt;

  async function choose(choice) {
    await respondPermission(prompt.id, choice);
    $permissionPrompt = null;
  }
</script>

<div class="card">
  <div class="head">
    <span class="icon">🔐</span>
    <span class="tool">{prompt.toolName}</span>
  </div>
  <pre class="description">{prompt.description}</pre>
  <div class="actions">
    <button class="allow" on:click={() => choose("allow_once")}>Allow once</button>
    <button class="always" on:click={() => choose("allow_always")}>Always allow</button>
    <button class="deny" on:click={() => choose("deny")}>Deny</button>
  </div>
</div>

<style>
  .card {
    background: var(--bg-card);
    border: 1px solid var(--accent);
    border-radius: 14px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-size: 13px;
  }

  .description {
    font-size: 12px;
    color: var(--text-dim);
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 180px;
    overflow-y: auto;
  }

  .actions {
    display: flex;
    gap: 8px;
  }

  .actions button {
    border-radius: 8px;
    padding: 6px 14px;
    font-size: 13px;
  }

  .allow {
    background: var(--accent);
    color: white;
  }

  .always {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .deny {
    background: rgba(242, 109, 109, 0.15);
    color: var(--danger);
  }
</style>
