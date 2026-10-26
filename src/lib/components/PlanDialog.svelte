<script lang="ts">
  import { fade, scale } from 'svelte/transition'
  import { t } from '$lib/i18n.svelte'
  import type { Preview } from '$lib/api'
  import Icon from './Icon.svelte'

  let {
    title,
    previews,
    busy = false,
    confirmLabel,
    onconfirm,
    oncancel,
  }: {
    title: string
    previews: Preview[]
    busy?: boolean
    confirmLabel?: string
    onconfirm: () => void
    oncancel: () => void
  } = $props()

  const total = $derived(previews.reduce((n, p) => n + p.changes, 0))
  let confirmBtn = $state<HTMLButtonElement>()

  $effect(() => {
    confirmBtn?.focus()
  })
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && !busy && oncancel()} />

<div class="scrim" transition:fade={{ duration: 160 }} onclick={() => !busy && oncancel()} role="presentation"></div>

<div class="dialog" role="dialog" aria-modal="true" aria-labelledby="plan-title" transition:scale={{ start: 0.96, duration: 200 }}>
  <header>
    <h2 id="plan-title">{title}</h2>
    <p>{t('plan.summary', { files: previews.length, changes: total })}</p>
  </header>

  <ul class="list">
    {#each previews.slice(0, 60) as p (p.file)}
      <li>
        <div class="file">
          <code>{p.file}</code>
          <span>{t('plan.changes', { n: p.changes })}</span>
        </div>
        {#if p.before || p.after}
          <div class="diff">
            <div class="old"><i>−</i><span>{p.before}</span></div>
            <div class="new"><i>+</i><span>{p.after}</span></div>
          </div>
        {/if}
      </li>
    {/each}
    {#if previews.length > 60}
      <li class="more">{t('plan.more', { n: previews.length - 60 })}</li>
    {/if}
  </ul>

  <div class="safe">
    <Icon name="shieldCheck" size={18} stroke={1.8} />
    <span>{t('plan.safety')}</span>
  </div>

  <footer>
    <button class="btn" onclick={oncancel} disabled={busy}>{t('plan.cancel')}</button>
    <button class="btn primary" bind:this={confirmBtn} onclick={onconfirm} disabled={busy}>
      {#if busy}<span class="spin" aria-hidden="true"></span>{/if}
      {busy ? t('plan.applying') : (confirmLabel ?? t('plan.confirm'))}
    </button>
  </footer>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 50;
    background: rgba(8, 8, 12, 0.55);
    backdrop-filter: blur(6px);
  }
  .dialog {
    position: fixed;
    left: 50%;
    top: 50%;
    translate: -50% -50%;
    width: min(640px, calc(100vw - 48px));
    max-height: calc(100vh - 120px);
    z-index: 51;
    display: flex;
    flex-direction: column;
    border-radius: 26px;
    background: rgba(26, 26, 32, 0.96);
    border: 1px solid rgba(255, 255, 255, 0.12);
    box-shadow: 0 40px 100px rgba(0, 0, 0, 0.6);
    overflow: hidden;
  }
  header {
    padding: 24px 26px 14px;
  }
  h2 {
    font-size: 21px;
    font-weight: 600;
    letter-spacing: -0.02em;
  }
  header p {
    margin-top: 6px;
    font-size: 14px;
    color: var(--text-2);
  }

  .list {
    list-style: none;
    overflow-y: auto;
    padding: 0 26px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.14) transparent;
  }
  li {
    padding: 12px 14px;
    border-radius: 14px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.06);
  }
  .file {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: baseline;
  }
  code,
  .diff {
    font-family: ui-monospace, 'Cascadia Code', 'SF Mono', Consolas, monospace;
  }
  code {
    font-size: 12.5px;
    color: #dcdce2;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .file span {
    font-size: 12px;
    color: var(--text-3);
    flex: none;
  }
  .diff {
    margin-top: 8px;
    font-size: 12px;
    line-height: 1.55;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .diff > div {
    display: flex;
    gap: 8px;
    padding: 3px 8px;
    border-radius: 7px;
    overflow-wrap: anywhere;
  }
  .diff i {
    font-style: normal;
    opacity: 0.7;
    flex: none;
  }
  .old {
    background: rgba(255, 122, 112, 0.09);
    color: #ffb4ad;
  }
  .new {
    background: rgba(111, 211, 154, 0.09);
    color: #a9ecc8;
  }
  .more {
    text-align: center;
    font-size: 12.5px;
    color: var(--text-3);
  }

  .safe {
    margin: 16px 26px 0;
    padding: 12px 14px;
    border-radius: 14px;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    line-height: 1.45;
    color: #cfe9dc;
    background: rgba(111, 211, 154, 0.08);
    border: 1px solid rgba(111, 211, 154, 0.18);
  }
  .safe :global(svg) {
    color: var(--green);
    flex: none;
  }

  footer {
    padding: 18px 26px 22px;
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }
  .btn:disabled {
    opacity: 0.6;
    cursor: default;
  }
  .spin {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 2px solid rgba(0, 0, 0, 0.2);
    border-top-color: #0a0a0d;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
