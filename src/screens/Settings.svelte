<script lang="ts">
  import { t } from '$lib/i18n.svelte'
  import { settings, RETENTION_OPTIONS, type Lang, type UiScale } from '$lib/settings.svelte'
  import { back, nav } from '$lib/nav.svelte'
  import { openLink, pickFolder, inTauri } from '$lib/native'
  import { updates, checkUpdates, installUpdate } from '$lib/updates.svelte'
  import { openBackupDir, friendlyError } from '$lib/api'
  import { exportDiagnostics } from '$lib/share'
  import { notify } from '$lib/toast.svelte'
  import { FOXIVO_URL } from '$lib/links'
  import Icon from '$lib/components/Icon.svelte'
  import Toggle from '$lib/components/Toggle.svelte'
  import Segmented from '$lib/components/Segmented.svelte'
  import logo from '$lib/assets/logo.png'

  const RULES_VERSION = '2026.10.26'

  const versionHint = $derived(
    updates.status === 'latest'
      ? t('update.latest')
      : updates.status === 'available'
        ? t('update.availableShort', { v: updates.version })
        : updates.status === 'downloading'
          ? t('update.downloading', { v: updates.version })
          : updates.status === 'error'
            ? t('update.error')
            : t('settings.versionLine', { rules: RULES_VERSION }),
  )

  const langs: { value: Lang; label: string }[] = [
    { value: 'tr', label: 'Türkçe' },
    { value: 'en', label: 'English' },
  ]
  const scales = $derived<{ value: UiScale; label: string }[]>([
    { value: 'compact', label: t('settings.scaleCompact') },
    { value: 'auto', label: t('settings.scaleAuto') },
    { value: 'large', label: t('settings.scaleLarge') },
  ])

  async function chooseBackupDir() {
    const dir = await pickFolder(t('settings.backupDir'))
    if (dir) settings.backupDir = dir
  }

  async function openDir() {
    try {
      await openBackupDir()
    } catch (e) {
      notify(friendlyError(e), 'error')
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && back()} />

<div class="page">
  <div class="col">
    <div class="hd">
      <button class="back" onclick={back} aria-label={t('settings.back')}>
        <Icon name="chevronLeft" size={18} stroke={2} />
      </button>
      <h1>{t('settings.title')}</h1>
      <span>{t('settings.autosave')}</span>
    </div>

    <div class="grid">
      <section class="card glass">
        <h2>{t('settings.general')}</h2>
        <div class="row">
          <div class="txt"><b>{t('settings.language')}</b><span>{t('settings.languageHint')}</span></div>
          <Segmented options={langs} bind:value={settings.lang} label={t('settings.language')} />
        </div>
        <div class="row">
          <div class="txt"><b>{t('settings.scale')}</b><span>{t('settings.scaleHint')}</span></div>
          <Segmented options={scales} bind:value={settings.uiScale} label={t('settings.scale')} />
        </div>
        <div class="row">
          <div class="txt"><b>{t('settings.closeToTray')}</b><span>{t('settings.closeToTrayHint')}</span></div>
          <Toggle bind:checked={settings.closeToTray} label={t('settings.closeToTray')} />
        </div>
        <div class="row">
          <div class="txt"><b>{t('settings.glow')}</b><span>{t('settings.glowHint')}</span></div>
          <Toggle bind:checked={settings.glow} label={t('settings.glow')} />
        </div>
        <div class="row">
          <div class="txt"><b>{t('settings.reduceMotion')}</b><span>{t('settings.reduceMotionHint')}</span></div>
          <Toggle bind:checked={settings.reduceMotion} label={t('settings.reduceMotion')} />
        </div>
      </section>

      <section class="card glass">
        <h2>{t('settings.safety')}</h2>
        <div class="row">
          <div class="txt"><b>{t('settings.approval')}</b><span>{t('settings.approvalHint')}</span></div>
          <span class="locked"><Icon name="lock" size={13} stroke={2} />{t('settings.alwaysOn')}</span>
        </div>
        <div class="row">
          <div class="txt"><b>{t('settings.autoBackup')}</b><span>{t('settings.autoBackupHint')}</span></div>
          <span class="locked"><Icon name="lock" size={13} stroke={2} />{t('settings.alwaysOn')}</span>
        </div>
        <div class="row">
          <div class="txt"><b>{t('settings.detectRunning')}</b><span>{t('settings.detectRunningHint')}</span></div>
          <span class="locked"><Icon name="lock" size={13} stroke={2} />{t('settings.alwaysOn')}</span>
        </div>
        <div class="row">
          <div class="txt"><b>{t('settings.verifyOnline')}</b><span>{t('settings.verifyOnlineHint')}</span></div>
          <Toggle bind:checked={settings.verifyOnline} label={t('settings.verifyOnline')} />
        </div>
        <div class="row">
          <div class="txt"><b>{t('settings.diagnostics')}</b><span>{t('settings.diagnosticsHint')}</span></div>
          <button class="pill" onclick={() => exportDiagnostics(nav.report)}>{t('settings.export')}</button>
        </div>
      </section>

      <section class="card glass">
        <h2>{t('settings.backups')}</h2>
        <div class="row">
          <div class="txt"><b>{t('settings.backupDir')}</b><span>{t('settings.backupDirHint')}</span></div>
          <button class="pill path" onclick={chooseBackupDir} title={settings.backupDir}>{settings.backupDir}</button>
        </div>
        <div class="row">
          <div class="txt"><b>{t('settings.retention')}</b><span>{t('settings.retentionHint')}</span></div>
          <label class="select">
            <select bind:value={settings.retentionDays} aria-label={t('settings.retention')}>
              {#each RETENTION_OPTIONS as n (n)}
                <option value={n}>{t('settings.days', { n })}</option>
              {/each}
            </select>
            <Icon name="chevrons" size={14} stroke={2} />
          </label>
        </div>
        <div class="row">
          <div class="txt"><b>{t('settings.openBackups')}</b><span>{t('settings.openBackupsHint')}</span></div>
          <button class="pill" onclick={openDir}>{t('settings.open')}</button>
        </div>
      </section>

      <section class="card glass">
        <h2>{t('settings.about')}</h2>
        <div class="row">
          <div class="txt">
            <b>{t('settings.version')} {__APP_VERSION__}</b>
            <span class:ok={updates.status === 'latest'} class:new={updates.status === 'available'}>{versionHint}</span>
          </div>
          {#if updates.status === 'available'}
            <button class="pill accent" onclick={installUpdate}>{t('update.install')}</button>
          {:else}
            <button class="pill" onclick={() => checkUpdates()} disabled={updates.status === 'checking' || updates.status === 'downloading' || !inTauri}>
              {updates.status === 'checking' ? t('update.checking') : t('update.check')}
            </button>
          {/if}
        </div>
        <div class="row">
          <div class="txt"><b>{t('settings.autoUpdate')}</b><span>{t('settings.autoUpdateHint')}</span></div>
          <Toggle bind:checked={settings.autoUpdate} label={t('settings.autoUpdate')} />
        </div>
        <div class="row">
          <div class="txt brand"><img src={logo} alt="" /><div><b>Foxivo</b><span>{t('settings.madeBy')}</span></div></div>
          <button class="pill" onclick={() => openLink(FOXIVO_URL)}>foxivo.com.tr <Icon name="external" size={12} stroke={2} /></button>
        </div>
      </section>
    </div>
  </div>
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
    overflow-y: auto;
    padding: calc(var(--bar-h) + 28px) 24px 110px;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.12) transparent;
  }
  .col {
    width: min(940px, 100%);
    margin: 0 auto;
  }
  .hd {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .back {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.06);
    color: #e5e5ea;
    transition: background 0.15s;
  }
  .back:hover {
    background: rgba(255, 255, 255, 0.12);
  }
  h1 {
    font-size: 34px;
    font-weight: 600;
    letter-spacing: -0.032em;
  }
  .hd > span {
    margin-left: auto;
    font-size: 13px;
    color: var(--text-3);
  }

  .grid {
    margin-top: 24px;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
  }
  .card {
    border-radius: 24px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    padding: 18px 8px 8px;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 20px 50px rgba(0, 0, 0, 0.25);
  }
  h2 {
    padding: 0 14px 8px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: var(--text-3);
  }
  .row {
    min-height: 60px;
    padding: 8px 14px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }
  h2 + .row {
    border-top: none;
  }
  .txt {
    min-width: 0;
  }
  .txt b {
    display: block;
    font-size: 14.5px;
    font-weight: 500;
    letter-spacing: -0.01em;
  }
  .txt span {
    display: block;
    margin-top: 2px;
    font-size: 12.5px;
    color: var(--text-3);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .brand img {
    width: 34px;
    height: 34px;
    border-radius: 10px;
    box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.1);
  }

  .pill {
    height: 32px;
    padding: 0 14px;
    border-radius: 99px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    font-weight: 600;
    color: #dcdce2;
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.1);
    flex: none;
    transition: background 0.15s;
  }
  button.pill:hover {
    background: rgba(255, 255, 255, 0.13);
    color: #fff;
  }
  .pill:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .pill.accent {
    color: #0a0a0d;
    background: var(--text);
    border-color: transparent;
  }
  .txt span.ok {
    color: var(--green);
  }
  .txt span.new {
    color: var(--blue);
  }
  .path {
    max-width: 210px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: block;
    line-height: 30px;
  }

  .select {
    position: relative;
    display: inline-flex;
    align-items: center;
    flex: none;
  }
  .select select {
    appearance: none;
    height: 32px;
    padding: 0 32px 0 14px;
    border-radius: 10px;
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    color: #e5e5ea;
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.1);
    cursor: pointer;
  }
  .select select option {
    background: #1c1c21;
    color: var(--text);
  }
  .select :global(svg) {
    position: absolute;
    right: 10px;
    color: var(--text-3);
    pointer-events: none;
  }

  .locked {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12.5px;
    font-weight: 600;
    color: var(--green);
    background: rgba(111, 211, 154, 0.12);
    padding: 6px 12px;
    border-radius: 99px;
    flex: none;
  }
</style>
