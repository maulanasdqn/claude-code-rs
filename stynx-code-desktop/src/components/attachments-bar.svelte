<script>
  import { pendingImages } from "../lib/stores.js";
  import { removeImage, dataUrl } from "../lib/images.js";
</script>

{#if $pendingImages.length > 0}
  <div class="strip">
    {#each $pendingImages as image (image.id)}
      <div class="thumb">
        <img src={dataUrl(image)} alt="attachment" />
        <button on:click={() => removeImage(image.id)} title="Remove">✕</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .strip {
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding: 2px;
  }

  .thumb {
    position: relative;
    flex-shrink: 0;
  }

  .thumb img {
    width: 54px;
    height: 54px;
    object-fit: cover;
    border-radius: 8px;
  }

  .thumb button {
    position: absolute;
    top: -6px;
    right: -6px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.7);
    color: white;
    font-size: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
