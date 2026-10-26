<script lang="ts">
  import { fade } from 'svelte/transition'
  import { toast } from '$lib/toast.svelte'
  import Icon from './Icon.svelte'

  const icons = { info: 'lock', success: 'check', error: 'alert' }
</script>

{#key toast.key}
  {#if toast.text}
    <div class="toast {toast.tone}" role="status" transition:fade={{ duration: 200 }}>
      <Icon name={icons[toast.tone]} size={16} stroke={2} />
      <span>{toast.text}</span>
    </div>
  {/if}
{/key}

<style>
  .toast {
    position: fixed;
    top: calc(var(--bar-h) + 10px);
    left: 50%;
    translate: -50% 0;
    max-width: min(560px, calc(100vw - 48px));
    padding: 12px 18px 12px 14px;
    border-radius: 16px;
    display: flex;
    align-items: center;
    gap: 10px;
    z-index: 60;
    font-size: 13.5px;
    line-height: 1.45;
    color: #ececf1;
    background: rgba(30, 30, 38, 0.94);
    border: 1px solid rgba(255, 255, 255, 0.12);
    backdrop-filter: blur(24px);
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.45);
  }
  .toast :global(svg) {
    flex: none;
  }
  .info :global(svg) {
    color: var(--text-2);
  }
  .success :global(svg) {
    color: var(--green);
  }
  .error {
    border-color: rgba(255, 138, 128, 0.35);
  }
  .error :global(svg) {
    color: var(--red);
  }
</style>
