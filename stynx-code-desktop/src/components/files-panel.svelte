<script>
  import { changes } from "../lib/stores.js";
  import DiffView from "./diff-view.svelte";

  let expanded = {};

  $: lastId = $changes[$changes.length - 1]?.id;

  function isOpen(change) {
    return expanded[change.id] ?? change.id === lastId;
  }

  function toggle(change) {
    expanded = { ...expanded, [change.id]: !isOpen(change) };
  }
</script>

<div class="panel">
  <div class="head">
    <span class="title">± Diff</span>
    {#if $changes.length > 0}
      <span class="count">{$changes.length}</span>
    {/if}
  </div>
  <div class="body">
    {#if $changes.length === 0}
      <div class="empty">No changes yet</div>
    {:else}
      {#each $changes as change (change.id)}
        <div class="card">
          <button class="card-head" on:click={() => toggle(change)}>
            <span class="chevron">{isOpen(change) ? "▾" : "▸"}</span>
            <span class="name">{change.name}</span>
            <span class="stats">
              {#if change.adds > 0}<span class="adds">+{change.adds}</span>{/if}
              {#if change.removes > 0}<span class="removes">-{change.removes}</span>{/if}
            </span>
            <span class="badge" class:add={change.kind === "file_write"}>
              {change.kind === "file_write" ? "A" : "M"}
            </span>
          </button>
          {#if isOpen(change)}
            <DiffView {change} />
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .panel {
    width: 420px;
    flex-shrink: 0;
    border-left: 1px solid var(--border);
    background: var(--bg-panel);
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
  }

  .title {
    font-weight: 600;
    font-size: 14px;
  }

  .count {
    font-size: 11px;
    color: var(--text-dim);
    background: var(--bg-card);
    border-radius: 999px;
    padding: 1px 8px;
  }

  .body {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .empty {
    color: var(--text-dim);
    font-size: 13px;
    text-align: center;
    margin-top: 40px;
  }

  .card {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 12px;
    overflow: hidden;
  }

  .card-head {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 10px;
    font-size: 12px;
    text-align: left;
  }

  .chevron {
    color: var(--text-dim);
    font-size: 10px;
  }

  .name {
    flex: 1;
    min-width: 0;
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .stats {
    display: flex;
    gap: 6px;
    font-family: ui-monospace, monospace;
    font-size: 11px;
  }

  .adds {
    color: var(--ok);
  }

  .removes {
    color: var(--danger);
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
</style>
