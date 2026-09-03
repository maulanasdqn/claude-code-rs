<script>
  import { mode, thinking } from "../../lib/stores.js";
  import { applyMode, applyThinking } from "../../lib/session-actions.js";

  const MODES = [
    { value: "normal", label: "Normal" },
    { value: "auto", label: "Auto" },
    { value: "plan", label: "Plan" },
    { value: "bypass", label: "Bypass" },
  ];
</script>

<section>
  <h3 class="side-title">Permission</h3>
  <label class="field">
    <span>Mode</span>
    <select value={$mode} on:change={(event) => applyMode(event.target.value)}>
      {#each MODES as option}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
  </label>
  <label class="check">
    <input
      type="checkbox"
      checked={$thinking}
      on:change={(event) => applyThinking(event.target.checked)}
    />
    Thinking
  </label>
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    font-size: 13px;
  }

  .field select {
    max-width: 150px;
  }

  .check {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }
</style>
