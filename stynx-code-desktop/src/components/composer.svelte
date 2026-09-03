<script>
  import { get } from "svelte/store";
  import { isStreaming, composerDraft, pendingImages } from "../lib/stores.js";
  import { send } from "../lib/messaging.js";
  import { cancel } from "../lib/api.js";
  import { historyUp, historyDown } from "../lib/prompt-history.js";
  import { imagesFromPaste, addImageFiles } from "../lib/images.js";
  import { addReferenceFromUrl } from "../lib/references.js";
  import AttachmentsBar from "./attachments-bar.svelte";
  import MentionPopup from "./mention-popup.svelte";

  let fileInput;

  function submit() {
    const text = $composerDraft;
    const images = get(pendingImages);
    $composerDraft = "";
    $pendingImages = [];
    send(text, images);
  }

  function onKeydown(event) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      submit();
    } else if (event.key === "ArrowUp") {
      const previous = historyUp($composerDraft);
      if (previous !== null) {
        event.preventDefault();
        $composerDraft = previous;
      }
    } else if (event.key === "ArrowDown") {
      const next = historyDown();
      if (next !== null) {
        event.preventDefault();
        $composerDraft = next;
      }
    }
  }

  function onPaste(event) {
    if (imagesFromPaste(event)) {
      event.preventDefault();
      return;
    }
    const text = event.clipboardData?.getData("text") ?? "";
    if (/^https?:\/\/\S+$/.test(text.trim())) {
      event.preventDefault();
      addReferenceFromUrl(text);
    }
  }

  function onAttach(event) {
    addImageFiles([...event.target.files]);
    event.target.value = "";
  }
</script>

<div class="composer">
  <MentionPopup />
  <AttachmentsBar />
  <div class="input-row">
    <textarea
      bind:value={$composerDraft}
      on:keydown={onKeydown}
      on:paste={onPaste}
      placeholder="Ask stynx…"
      rows="3"
      spellcheck="false"
    ></textarea>
    <div class="actions">
      <button class="attach" on:click={() => fileInput.click()} title="Attach image">📎</button>
      <input
        type="file"
        accept="image/*"
        multiple
        bind:this={fileInput}
        on:change={onAttach}
        hidden
      />
      {#if $isStreaming}
        <button class="stop" on:click={() => cancel()} title="Stop">■</button>
      {:else}
        <button
          class="go"
          on:click={submit}
          disabled={!$composerDraft.trim() && $pendingImages.length === 0}
          title="Send">↑</button
        >
      {/if}
    </div>
  </div>
</div>

<style>
  .composer {
    margin: 8px 16px 14px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .input-row {
    display: flex;
    align-items: flex-end;
    gap: 10px;
  }

  textarea {
    flex: 1;
    background: none;
    border: none;
    resize: none;
    max-height: 200px;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .actions button {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    font-size: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .attach {
    font-size: 14px;
    color: var(--text-dim);
  }

  .attach:hover {
    background: var(--accent-soft);
  }

  .go {
    background: var(--accent);
    color: white;
  }

  .go:disabled {
    background: var(--bg-panel);
    color: var(--text-dim);
  }

  .stop {
    background: var(--danger);
    color: white;
    font-size: 12px;
  }
</style>
