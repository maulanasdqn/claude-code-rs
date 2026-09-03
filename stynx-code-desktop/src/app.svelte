<script>
  import { onMount } from "svelte";
  import Sidebar from "./components/sidebar.svelte";
  import Chat from "./components/chat.svelte";
  import TopBar from "./components/top-bar.svelte";
  import FilesPanel from "./components/files-panel.svelte";
  import { attachEngineEvents } from "./lib/engine-events.js";
  import { openWorkspace } from "./lib/session-actions.js";
  import { status, showFiles } from "./lib/stores.js";

  let bootError = "";

  onMount(async () => {
    await attachEngineEvents();
    await open(null, null);
  });

  async function open(path, provider) {
    bootError = "";
    try {
      await openWorkspace(path, provider);
    } catch (error) {
      bootError = String(error);
      $status = "Init failed";
    }
  }
</script>

<div class="layout">
  <Sidebar onOpenWorkspace={open} />
  <main>
    <TopBar />
    {#if bootError}
      <div class="boot-error">Could not start the engine: {bootError}</div>
    {/if}
    <div class="content">
      <Chat />
      {#if $showFiles}
        <FilesPanel />
      {/if}
    </div>
  </main>
</div>

<style>
  .layout {
    display: flex;
    height: 100%;
  }

  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .content {
    flex: 1;
    display: flex;
    min-height: 0;
  }

  .boot-error {
    margin: 12px 16px 0;
    padding: 10px 14px;
    border-radius: 10px;
    background: rgba(242, 109, 109, 0.12);
    border: 1px solid rgba(242, 109, 109, 0.4);
    color: var(--danger);
    font-size: 13px;
  }
</style>
