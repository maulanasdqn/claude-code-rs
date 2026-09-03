<script>
  import { info, status, isStreaming, showFiles, changes } from "../lib/stores.js";
</script>

<div class="bar">
  <span class="project">{$info?.projectName ?? "Stynx"}</span>
  <span class="spacer"></span>
  <span class="pill">
    {#if $isStreaming}
      <span class="spinner"></span>
    {:else}
      <span class="dot" class:bad={$status === "Init failed"}></span>
    {/if}
    {$status}
  </span>
  <button class="files" class:active={$showFiles} on:click={() => ($showFiles = !$showFiles)}>
    ± Diff{#if $changes.length > 0}&nbsp;· {$changes.length}{/if}
  </button>
</div>

<style>
  .bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    border-bottom: 1px solid var(--border);
  }

  .project {
    font-weight: 600;
    font-size: 14px;
  }

  .spacer {
    flex: 1;
  }

  .pill {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 12px;
    color: var(--text-dim);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 999px;
    padding: 5px 14px;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--ok);
  }

  .dot.bad {
    background: var(--danger);
  }

  .spinner {
    width: 10px;
    height: 10px;
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

  .files {
    font-size: 12px;
    color: var(--text-dim);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 5px 10px;
  }

  .files.active {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-soft);
  }
</style>
