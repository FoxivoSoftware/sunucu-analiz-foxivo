<script lang="ts">
  import { onMount } from 'svelte'
  import { t } from '$lib/i18n.svelte'
  import { settings } from '$lib/settings.svelte'
  import { nav, go, back } from '$lib/nav.svelte'
  import { isMac, minimize, toggleMaximize, close, isMaximized, onResize } from '$lib/native'
  import Icon from './Icon.svelte'
  import Segmented from './Segmented.svelte'
  import logo from '$lib/assets/logo.png'

  const langs = [
    { value: 'tr' as const, label: 'TR' },
    { value: 'en' as const, label: 'EN' },
  ]

  let maximized = $state(false)

  onMount(() => {
    const sync = async () => (maximized = await isMaximized())
    sync()
    const unlisten = onResize(sync)
    return () => {
      unlisten.then((off) => off())
    }
  })

  const toggleSettings = () => (nav.screen === 'settings' ? back() : go('settings'))
</script>

<header class="bar" class:mac={isMac} data-tauri-drag-region>
  <div class="side" data-tauri-drag-region></div>

  <button class="title" onclick={() => go('home')} title={t('titlebar.home')}>
    <img src={logo} alt="" draggable="false" />
    <span>{t('app.name')}</span>
  </button>

  <div class="side right">
    <Segmented options={langs} bind:value={settings.lang} label={t('titlebar.language')} />
    <button
      class="icon-btn"
      class:active={nav.screen === 'settings'}
      onclick={toggleSettings}
      aria-label={t('titlebar.settings')}
      title={t('titlebar.settings')}
    >
      <Icon name="gear" size={17} stroke={1.6} />
    </button>

    {#if !isMac}
      <div class="controls">
        <button class="wb" onclick={minimize} aria-label={t('titlebar.minimize')} title={t('titlebar.minimize')}>
          <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 5h8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" /></svg>
        </button>
        <button class="wb" onclick={toggleMaximize} aria-label={t(maximized ? 'titlebar.restore' : 'titlebar.maximize')} title={t(maximized ? 'titlebar.restore' : 'titlebar.maximize')}>
          {#if maximized}
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2">
              <rect x="1" y="3" width="6" height="6" rx="1.4" />
              <path d="M3.2 3V2.4A1.4 1.4 0 0 1 4.6 1H7.6A1.4 1.4 0 0 1 9 2.4v3a1.4 1.4 0 0 1-1.4 1.4H7" />
            </svg>
          {:else}
            <svg width="10" height="10" viewBox="0 0 10 10"><rect x="1.2" y="1.2" width="7.6" height="7.6" rx="1.8" fill="none" stroke="currentColor" stroke-width="1.2" /></svg>
          {/if}
        </button>
        <button class="wb close" onclick={close} aria-label={t('titlebar.close')} title={t('titlebar.close')}>
          <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1.5 1.5l7 7M8.5 1.5l-7 7" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" /></svg>
        </button>
      </div>
    {/if}
  </div>
</header>

<style>
  .bar {
    position: fixed;
    inset: 0 0 auto 0;
    height: var(--bar-h);
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding: 0 12px 0 16px;
    z-index: 20;
    user-select: none;
    /* content scrolls underneath and fades out instead of being cut at a hard line */
    background: linear-gradient(to bottom, rgba(18, 18, 21, 0.94) 55%, rgba(18, 18, 21, 0));
  }
  .mac {
    padding-left: 84px;
  }

  .side {
    height: 100%;
  }
  .right {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
  }

  .title {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 36px;
    padding: 0 14px 0 6px;
    border-radius: 12px;
    font-size: 14px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: #e8e8ed;
    transition: background 0.2s;
  }
  .title:hover {
    background: rgba(255, 255, 255, 0.06);
  }
  .title img {
    width: 26px;
    height: 26px;
    border-radius: 8px;
    display: block;
    box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.1), 0 2px 8px rgba(0, 0, 0, 0.4);
  }

  .icon-btn {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: #c7c7cc;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--line);
    transition: background 0.15s, color 0.15s;
  }
  .icon-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: var(--text);
  }
  .icon-btn.active {
    background: rgba(255, 255, 255, 0.18);
    color: #fff;
  }

  .controls {
    display: flex;
    gap: 2px;
    margin-left: 6px;
    padding: 3px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.07);
  }
  .wb {
    width: 34px;
    height: 28px;
    border-radius: 9px;
    display: grid;
    place-items: center;
    color: #b4b4bc;
    transition: background 0.15s, color 0.15s;
  }
  .wb:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }
  .wb.close:hover {
    background: #e5484d;
    color: #fff;
  }
</style>
