<script>
  import { tick } from "svelte";
  import FeedItem from "./feed-item.svelte";
  import PermissionCard from "./permission-card.svelte";
  import QuestionCard from "./question-card.svelte";
  import QueuePanel from "./queue-panel.svelte";
  import Composer from "./composer.svelte";
  import { feed, permissionPrompt, question, isStreaming } from "../lib/stores.js";

  let scroller;

  $: $feed, $permissionPrompt, $question, $isStreaming, scrollToBottom();

  async function scrollToBottom() {
    await tick();
    if (scroller) scroller.scrollTop = scroller.scrollHeight;
  }
</script>

<div class="chat">
  <div class="transcript" bind:this={scroller}>
    {#each $feed as item (item.id)}
      <FeedItem {item} />
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

  <QueuePanel />
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
</style>
