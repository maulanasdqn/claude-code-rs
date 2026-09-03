<script>
  import { respondAskUser } from "../lib/api.js";
  import { question as questionStore } from "../lib/stores.js";

  export let question;

  let draft = "";
  let selections = {};

  async function answer(text) {
    await respondAskUser(question.id, text);
    $questionStore = null;
  }

  async function skip() {
    await respondAskUser(question.id, "(skipped)");
    $questionStore = null;
  }

  function toggle(qIndex, label) {
    const current = selections[qIndex] ?? [];
    selections = {
      ...selections,
      [qIndex]: current.includes(label)
        ? current.filter((l) => l !== label)
        : [...current, label],
    };
  }

  function sendMulti(qIndex) {
    const picked = selections[qIndex] ?? [];
    if (picked.length > 0) answer(picked.join(", "));
  }
</script>

<div class="card">
  {#if question.qa}
    {#each question.qa as qa, qIndex}
      <div class="qa">
        <div class="header">{qa.header}</div>
        <div class="question">{qa.question}</div>
        <div class="options">
          {#each qa.options as option}
            {#if qa.multiSelect}
              <label class="option check">
                <input
                  type="checkbox"
                  checked={(selections[qIndex] ?? []).includes(option.label)}
                  on:change={() => toggle(qIndex, option.label)}
                />
                <span>
                  <strong>{option.label}</strong>
                  {#if option.description}<em>{option.description}</em>{/if}
                </span>
              </label>
            {:else}
              <button class="option" on:click={() => answer(option.label)}>
                <strong>{option.label}</strong>
                {#if option.description}<em>{option.description}</em>{/if}
              </button>
            {/if}
          {/each}
        </div>
        {#if qa.multiSelect}
          <button class="send" on:click={() => sendMulti(qIndex)}>Send selection</button>
        {/if}
      </div>
    {/each}
  {:else}
    <div class="question">{question.question}</div>
    <textarea bind:value={draft} rows="3" placeholder="Your answer…"></textarea>
    <div class="actions">
      <button class="send" on:click={() => answer(draft)} disabled={!draft.trim()}>Send</button>
      <button class="skip" on:click={skip}>Skip</button>
    </div>
  {/if}
</div>

<style>
  .card {
    background: var(--bg-card);
    border: 1px solid var(--accent);
    border-radius: 14px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .header {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--accent);
    margin-bottom: 4px;
  }

  .question {
    font-size: 14px;
    white-space: pre-wrap;
  }

  .options {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 8px;
  }

  .option {
    text-align: left;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 8px 12px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 13px;
  }

  .option:hover {
    border-color: var(--accent);
  }

  .option em {
    color: var(--text-dim);
    font-style: normal;
    font-size: 12px;
  }

  .option.check {
    flex-direction: row;
    align-items: flex-start;
    gap: 8px;
    cursor: pointer;
  }

  .option.check span {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .actions {
    display: flex;
    gap: 8px;
  }

  .send {
    background: var(--accent);
    color: white;
    border-radius: 8px;
    padding: 6px 14px;
    font-size: 13px;
    align-self: flex-start;
  }

  .send:disabled {
    opacity: 0.5;
  }

  .skip {
    color: var(--text-dim);
    font-size: 13px;
  }
</style>
