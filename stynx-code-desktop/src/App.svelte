<script>
  import { onMount } from "svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import Chat from "./components/Chat.svelte";
  import { initSession, listSessions } from "./lib/api.js";
  import { attachEngineEvents } from "./lib/events.js";
  import { info, status, sessions, interns } from "./lib/stores.js";

  let bootError = "";

  onMount(async () => {
    await attachEngineEvents();
    await openWorkspace(null, null);
  });

  async function openWorkspace(path, provider) {
    bootError = "";
    $status = "Starting…";
    try {
      const result = await initSession(path, provider);
      $info = result;
      $interns = result.interns;
      $status = "Ready";
      $sessions = await listSessions();
    } catch (error) {
      bootError = String(error);
      $status = "Init failed";
    }
  }

</script>

<div class="layout">
  <Sidebar {openWorkspace} />
  <main>
    {#if bootError}
      <div class="boot-error">Could not start the engine: {bootError}</div>
    {/if}
    <Chat />
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
