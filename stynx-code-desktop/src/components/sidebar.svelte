<script>
  import WorkspaceSection from "./sidebar/workspace-section.svelte";
  import SessionsSection from "./sidebar/sessions-section.svelte";
  import ModelSection from "./sidebar/model-section.svelte";
  import PermissionSection from "./sidebar/permission-section.svelte";
  import InternsSection from "./sidebar/interns-section.svelte";

  export let onOpenWorkspace;
</script>

<aside>
  <WorkspaceSection {onOpenWorkspace} />
  <SessionsSection />
  <ModelSection {onOpenWorkspace} />
  <PermissionSection />
  <InternsSection />
</aside>

<style>
  aside {
    width: 270px;
    flex-shrink: 0;
    background: var(--bg-panel);
    border-right: 1px solid var(--border);
    overflow-y: auto;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
</style>
