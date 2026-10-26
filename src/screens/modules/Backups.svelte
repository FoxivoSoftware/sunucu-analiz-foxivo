<script lang="ts">
  import { onMount } from 'svelte'
  import { t } from '$lib/i18n.svelte'
  import { settings } from '$lib/settings.svelte'
  import { nav, rescan } from '$lib/nav.svelte'
  import { notify } from '$lib/toast.svelte'
  import { listBackups, restoreBackup, openBackupDir, friendlyError, type Backup } from '$lib/api'
  import Icon from '$lib/components/Icon.svelte'
  import ActionBar from '$lib/components/ActionBar.svelte'
  import PlanDialog from '$lib/components/PlanDialog.svelte'

  let backups = $state<Backup[]>([])
  let loading = $state(true)
  let target = $state<Backup | null>(null)
  let busy = $state(false)

  async function load() {
    if (!nav.path) return
    try {
      backups = await listBackups(nav.path)
    } finally {
      loading = false
    }
  }

  onMount(load)

  const when = (secs: number) =>
    new Date(secs * 1000).toLocaleString(settings.lang, { day: 'numeric', month: 'long', hour: '2-digit', minute: '2-digit' })

  async function restore() {
    if (!nav.path || !target) return
    busy = true
    try {
      await restoreBackup(nav.path, target.id)
      notify(t('backups.restored', { n: target.files.length }), 'success')
      target = null
      await Promise.all([load(), rescan()])
    } catch (e) {
      notify(friendlyError(e), 'error')
    } finally {
      busy = false
    }
  }

  async function openDir() {
    try {
      await openBackupDir()
    } catch (e) {
      notify(friendlyError(e), 'error')
    }
  }
</script>

{#if loading}
  <p class="hint">{t('backups.loading')}</p>
{:else if backups.length}
  <ul class="items">
    {#each backups as b (b.id)}
      <li class="item">
        <div class="item-row">
          <span class="ico"><Icon name="history" size={18} stroke={1.7} /></span>
          <div class="item-text">
            <b>{t(`backups.label.${b.label}`)}</b>
            <span class="mono">{when(b.created)} · {t('backups.files', { n: b.files.length })}</span>
          </div>
          {#if b.status === 'restored'}
            <span class="sev info"><i></i>{t('backups.wasRestored')}</span>
          {:else}
            <button class="btn small" onclick={() => (target = b)}>{t('backups.restore')}</button>
          {/if}
        </div>
      </li>
    {/each}
  </ul>
{:else}
  <div class="empty">
    <span class="empty-icon"><Icon name="history" size={28} stroke={1.6} /></span>
    <b>{t('backups.emptyTitle')}</b>
    <p>{t('backups.emptyText')}</p>
  </div>
{/if}

<p class="hint">{t('backups.where', { dir: settings.backupDir, days: settings.retentionDays })}</p>

<ActionBar text={t('backups.always')}>
  <button class="btn" onclick={openDir}>
    <Icon name="folderOpen" size={16} stroke={1.8} />
    {t('backups.openDir')}
  </button>
</ActionBar>

{#if target}
  <PlanDialog
    title={t('backups.confirmTitle')}
    previews={target.files.map((f) => ({ file: f, changes: 1, before: '', after: '' }))}
    {busy}
    confirmLabel={t('backups.restore')}
    onconfirm={restore}
    oncancel={() => (target = null)}
  />
{/if}

<style>
  .ico {
    width: 36px;
    height: 36px;
    border-radius: 12px;
    display: grid;
    place-items: center;
    color: #e5e5ea;
    background: rgba(255, 255, 255, 0.06);
    flex: none;
  }
</style>
