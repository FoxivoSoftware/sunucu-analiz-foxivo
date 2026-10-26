<script lang="ts">
  import { fade } from 'svelte/transition'
  import { t } from '$lib/i18n.svelte'
  import { updates, installUpdate } from '$lib/updates.svelte'
  import Icon from './Icon.svelte'

  const visible = $derived(
    !updates.dismissed && (updates.status === 'available' || updates.status === 'downloading'),
  )
</script>

{#if visible}
  <div class="banner" role="status" transition:fade={{ duration: 200 }}>
    <span class="spark"><Icon name="download" size={15} stroke={2} /></span>
    {#if updates.status === 'downloading'}
      <span class="text">{t('update.downloading', { v: updates.version })}</span>
      <span class="bar"><i style:width="{Math.round(updates.progress * 100)}%"></i></span>
    {:else}
      <span class="text">{t('update.available', { v: updates.version })}</span>
      <button class="btn primary small" onclick={installUpdate}>{t('update.install')}</button>
      <button class="close" onclick={() => (updates.dismissed = true)} aria-label={t('update.later')} title={t('update.later')}>
        <Icon name="x" size={14} stroke={2} />
      </button>
    {/if}
  </div>
{/if}

<style>
  .banner {
    position: fixed;
    top: calc(var(--bar-h) + 6px);
    left: 50%;
    translate: -50% 0;
    z-index: 25;
    height: 44px;
    padding: 0 6px 0 8px;
    border-radius: 999px;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13.5px;
    color: #ececf1;
    background: rgba(30, 30, 38, 0.94);
    border: 1px solid rgba(141, 180, 255, 0.3);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.45), 0 0 30px rgba(141, 180, 255, 0.12);
    white-space: nowrap;
  }
  .spark {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: var(--blue);
    background: rgba(141, 180, 255, 0.14);
    flex: none;
  }
  .text {
    font-weight: 500;
  }
  .bar {
    width: 140px;
    height: 6px;
    margin-right: 12px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.1);
    overflow: hidden;
  }
  .bar i {
    display: block;
    height: 100%;
    background: var(--blue);
    transition: width 0.3s;
  }
  .close {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: var(--text-2);
  }
  .close:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text);
  }
</style>
