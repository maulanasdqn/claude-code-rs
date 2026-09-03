<script>
  import {
    info,
    status,
    tokens,
    sessions,
    interns,
    mode,
    thinking,
    isStreaming,
    feed,
    resetTranscript,
    nextId,
  } from "../lib/stores.js";
  import {
    setMode,
    setThinking,
    setModel,
    setProviderKey,
    listSessions,
    loadSession,
    newSession,
    deleteSession,
  } from "../lib/api.js";

  export let openWorkspace;

  let workspaceDraft = "";
  let modelDraft = "";
  let keyEntryFor = null;
  let keyDraft = "";

  $: if ($info) {
    workspaceDraft = $info.workspacePath;
    modelDraft = $info.modelId;
  }

  const MODES = [
    { value: "normal", label: "Normal" },
    { value: "auto", label: "Auto" },
    { value: "plan", label: "Plan" },
    { value: "bypass", label: "Bypass" },
  ];

  async function applyMode(event) {
    $mode = event.target.value;
    await setMode($mode);
  }

  async function applyThinking(event) {
    $thinking = event.target.checked;
    await setThinking($thinking);
  }

  async function applyModel() {
    const id = await setModel(modelDraft.trim());
    if ($info) $info = { ...$info, modelId: id };
  }

  async function switchProvider(event) {
    await openWorkspace($info?.workspacePath ?? null, event.target.value);
  }

  async function open(id) {
    if ($isStreaming) return;
    const turns = await loadSession(id);
    resetTranscript();
    $feed = turns.map((turn) => ({ id: nextId(), role: turn.role, text: turn.text }));
    $status = "Loaded session";
  }

  async function startNew() {
    if ($isStreaming) return;
    await newSession();
    resetTranscript();
    $status = "Ready";
  }

  async function remove(id) {
    await deleteSession(id);
    $sessions = await listSessions();
  }

  async function saveKey(intern) {
    const value = keyDraft.trim();
    keyEntryFor = null;
    keyDraft = "";
    if (!value) return;
    $interns = await setProviderKey(intern.keyEnv, value);
  }
</script>

<aside>
  <section>
    <h3>Workspace</h3>
    <div class="row">
      <input bind:value={workspaceDraft} placeholder="/path/to/project" spellcheck="false" />
      <button
        class="primary"
        on:click={() => openWorkspace(workspaceDraft.trim() || null, null)}
        disabled={$isStreaming}>Open</button
      >
    </div>
    {#if $info}
      <div class="dim">{$info.projectName}</div>
    {/if}
  </section>

  <section>
    <h3>Sessions</h3>
    <button class="ghost" on:click={startNew} disabled={$isStreaming}>＋ New conversation</button>
    {#each $sessions as summary (summary.id)}
      <div class="session-row">
        <button class="session" on:click={() => open(summary.id)}>
          <span class="title">{summary.title || "Untitled"}</span>
          <span class="dim">{summary.messageCount} messages</span>
        </button>
        <button class="delete" title="Delete" on:click={() => remove(summary.id)}>✕</button>
      </div>
    {:else}
      <div class="dim">No saved sessions</div>
    {/each}
  </section>

  <section>
    <h3>Model</h3>
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
          <select bind:value={modelDraft} on:change={applyModel}>
            {#each $info.claudeModels as id}
              <option value={id}>{id}</option>
            {/each}
          </select>
        </label>
      {:else}
        <label class="field">
          <span>Model id</span>
          <input bind:value={modelDraft} on:change={applyModel} spellcheck="false" />
        </label>
      {/if}
    {/if}
    <div class="dim">Status: {$status}</div>
    <div class="dim">Tokens: in {$tokens.input} · out {$tokens.output}</div>
  </section>

  <section>
    <h3>Permission</h3>
    <label class="field">
      <span>Mode</span>
      <select value={$mode} on:change={applyMode}>
        {#each MODES as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </label>
    <label class="check">
      <input type="checkbox" checked={$thinking} on:change={applyThinking} />
      Thinking
    </label>
  </section>

  <section>
    <h3>Interns</h3>
    {#each $interns as intern (intern.name)}
      <div class="intern">
        <div class="intern-head">
          <span class="mono">{intern.name}</span>
          {#if intern.available}
            <span class="ok">ready</span>
          {:else}
            <button class="ghost small" on:click={() => (keyEntryFor = intern.name)}>
              add key
            </button>
          {/if}
        </div>
        <div class="dim">{intern.provider} · {intern.model}</div>
        {#if keyEntryFor === intern.name}
          <div class="row">
            <input
              bind:value={keyDraft}
              placeholder={intern.keyEnv}
              type="password"
              spellcheck="false"
            />
            <button class="primary" on:click={() => saveKey(intern)}>Save</button>
          </div>
        {/if}
      </div>
    {:else}
      <div class="dim">No interns configured</div>
    {/each}
  </section>
</aside>

<style>
  aside {
    width: 270px;
    flex-shrink: 0;
    background: var(--bg-panel);
    border-right: 1px solid var(--border);
    overflow-y: auto;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  h3 {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-dim);
    margin-bottom: 8px;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .row {
    display: flex;
    gap: 6px;
  }

  .row input {
    flex: 1;
    min-width: 0;
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

  .check {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }

  .dim {
    color: var(--text-dim);
    font-size: 12px;
  }

  .mono {
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-size: 12px;
  }

  .ok {
    color: var(--ok);
    font-size: 11px;
  }

  .primary {
    background: var(--accent);
    color: white;
    border-radius: 8px;
    padding: 6px 12px;
    font-size: 13px;
  }

  .primary:disabled {
    opacity: 0.5;
  }

  .ghost {
    text-align: left;
    color: var(--text);
    font-size: 13px;
    padding: 5px 8px;
    border-radius: 8px;
  }

  .ghost:hover {
    background: var(--accent-soft);
  }

  .ghost.small {
    font-size: 11px;
    color: var(--accent);
    padding: 2px 6px;
  }

  .session-row {
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

  .session .title {
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

  .intern {
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .intern-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
</style>
