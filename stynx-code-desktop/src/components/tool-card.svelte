<script>
  export let tool;

  let expanded = false;

  const ICONS = {
    bash: "❯_",
    read: "📄",
    file_write: "📝",
    file_edit: "✏️",
    glob: "🔍",
    grep: "🔍",
    web_fetch: "🌐",
    web_search: "🌐",
    todo_write: "☑️",
    todo_read: "☑️",
  };

  $: icon = ICONS[tool.name] ?? (tool.name.startsWith("delegate_to_") ? "👥" : "⚙️");
</script>

<button class="card" class:error={tool.isError} on:click={() => (expanded = !expanded)}>
  <span class="icon">{icon}</span>
  <span class="text">
    <span class="title">{tool.title}</span>
    <span class="name">
      {tool.name}{#if tool.subtitle}
        · {tool.subtitle}{/if}
    </span>
  </span>
  {#if tool.badge}
    <span class="badge" class:add={tool.badge === "A"}>{tool.badge}</span>
  {/if}
  {#if tool.running}
    <span class="spinner"></span>
  {:else if tool.isError}
    <span class="mark bad">✕</span>
  {:else}
    <span class="mark ok">✓</span>
  {/if}
</button>
{#if expanded && tool.detail}
  <pre class="detail">{tool.detail}</pre>
{/if}

<style>
  .card {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    text-align: left;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 9px 12px;
  }

  .card.error {
    border-color: rgba(242, 109, 109, 0.4);
  }

  .icon {
    font-size: 13px;
    width: 26px;
    text-align: center;
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .title {
    font-size: 13px;
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .name {
    font-size: 11px;
    color: var(--text-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .badge {
    font-size: 10px;
    font-weight: 700;
    color: white;
    background: #4f8ef7;
    border-radius: 4px;
    padding: 1px 5px;
  }

  .badge.add {
    background: var(--ok);
  }

  .mark.ok {
    color: var(--ok);
  }

  .mark.bad {
    color: var(--danger);
  }

  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .detail {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 10px;
    font-size: 12px;
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 260px;
    overflow-y: auto;
  }
</style>
