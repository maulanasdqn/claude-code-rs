<script>
  import { composerDraft } from "../lib/stores.js";
  import { diffQuote } from "../lib/diff.js";

  export let change;

  let selected = new Set();

  function toggle(id) {
    selected = new Set(selected.has(id) ? [...selected].filter((s) => s !== id) : [...selected, id]);
  }

  function quote() {
    const lines = (change.diff ?? []).filter((line) => selected.has(line.id));
    if (lines.length === 0) return;
    const block = diffQuote(change.path, lines);
    $composerDraft = $composerDraft ? `${$composerDraft}\n${block}` : block;
    selected = new Set();
  }
</script>

<div class="diff">
  {#if selected.size > 0}
    <button class="quote" on:click={quote}>
      Quote {selected.size} line{selected.size === 1 ? "" : "s"} into composer
    </button>
  {/if}
  {#if change.diff}
    {#each change.diff as line (line.id)}
      <button
        class="line {line.kind}"
        class:selected={selected.has(line.id)}
        on:click={() => toggle(line.id)}
      >
        <span class="num">{line.oldNumber ?? ""}</span>
        <span class="num">{line.newNumber ?? ""}</span>
        <span class="sign">{line.kind === "added" ? "+" : line.kind === "removed" ? "-" : " "}</span>
        <span class="code">{line.text}</span>
      </button>
    {/each}
  {:else if change.lines}
    {#each change.lines as text, index}
      <div class="line added">
        <span class="num"></span>
        <span class="num">{index + 1}</span>
        <span class="sign">+</span>
        <span class="code">{text}</span>
      </div>
    {/each}
  {/if}
</div>

<style>
  .diff {
    display: flex;
    flex-direction: column;
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-size: 12px;
    padding: 6px 0;
    overflow-x: auto;
  }

  .quote {
    align-self: flex-start;
    margin: 4px 10px 6px;
    padding: 3px 10px;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 11px;
    font-weight: 600;
  }

  .line {
    display: flex;
    gap: 8px;
    padding: 1px 10px;
    text-align: left;
    white-space: pre;
    width: 100%;
  }

  .line.added {
    background: rgba(95, 213, 138, 0.1);
    color: var(--ok);
  }

  .line.removed {
    background: rgba(242, 109, 109, 0.1);
    color: var(--danger);
  }

  .line.context {
    color: var(--text-dim);
  }

  .line.selected {
    outline: 1px solid var(--accent);
    outline-offset: -1px;
    border-radius: 3px;
  }

  .num {
    width: 30px;
    flex-shrink: 0;
    text-align: right;
    color: var(--text-dim);
    opacity: 0.6;
    user-select: none;
  }

  .sign {
    width: 10px;
    flex-shrink: 0;
    user-select: none;
  }

  .code {
    flex: 1;
  }
</style>
