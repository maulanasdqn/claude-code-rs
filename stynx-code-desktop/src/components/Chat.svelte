<script>
  import { tick } from "svelte";
  import FeedItemView from "./FeedItemView.svelte";
  import PermissionCard from "./PermissionCard.svelte";
  import QuestionCard from "./QuestionCard.svelte";
  import Composer from "./Composer.svelte";
  import { feed, permissionPrompt, question, isStreaming, messageQueue } from "../lib/stores.js";

  let scroller;

  $: $feed, $permissionPrompt, $question, $isStreaming, scrollToBottom();

  async function scrollToBottom() {
    await tick();
    if (scroller) scroller.scrollTop = scroller.scrollHeight;
  }

  function dequeue(index) {
    $messageQueue = $messageQueue.filter((_, i) => i !== index);
  }
</script>

<div class="chat">
  <div class="transcript" bind:this={scroller}>
    {#each $feed as item (item.id)}
      <FeedItemView {item} />
    {/each}
    {#if $permissionPrompt}
      <PermissionCard prompt={$permissionPrompt} />
    {/if}
    {#if $question}
      <QuestionCard question={$question} />
    {/if}
    {#if $isStreaming}
      <div class="typing"><span></span><span></span><span></span></div>
    {/if}
  </div>

  {#if $messageQueue.length > 0}
    <div class="queue">
      <div class="queue-head">Queue · {$messageQueue.length}</div>
      {#each $messageQueue as queued, index}
        <div class="queued">
          <span>{queued.text || "(image)"}</span>
          <button on:click={() => dequeue(index)}>✕</button>
        </div>
      {/each}
    </div>
  {/if}

  <Composer />
</div>

<style>
  .chat {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    position: relative;
  }

  .transcript {
    flex: 1;
    overflow-y: auto;
    padding: 20px 22px 30px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .typing {
    display: flex;
    gap: 4px;
    padding: 4px 2px;
  }

  .typing span {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text-dim);
    animation: pulse 1.2s infinite ease-in-out;
  }

  .typing span:nth-child(2) {
    animation-delay: 0.15s;
  }

  .typing span:nth-child(3) {
    animation-delay: 0.3s;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 0.25;
      transform: translateY(0);
    }
    50% {
      opacity: 1;
      transform: translateY(-3px);
    }
  }

  .queue {
    position: absolute;
    right: 16px;
    bottom: 120px;
    width: 260px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  }

  .queue-head {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-dim);
  }

  .queued {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    font-size: 12px;
    background: var(--bg-panel);
    border-radius: 8px;
    padding: 6px 8px;
  }

  .queued span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .queued button {
    color: var(--text-dim);
  }
</style>
