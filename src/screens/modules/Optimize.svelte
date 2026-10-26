<script lang="ts">
  import { untrack } from 'svelte'
  import { SvelteSet } from 'svelte/reactivity'
  import { t } from '$lib/i18n.svelte'
  import { notify } from '$lib/toast.svelte'
  import { FixFlow } from '$lib/fixflow.svelte'
  import type { Report } from '$lib/api'
  import Icon from '$lib/components/Icon.svelte'
  import Check from '$lib/components/Check.svelte'
  import ActionBar from '$lib/components/ActionBar.svelte'
  import PlanDialog from '$lib/components/PlanDialog.svelte'

  let { report }: { report: Report } = $props()

  const applicable = $derived(report.optimize.filter((s) => s.applicable))
  const manual = $derived(report.optimize.filter((s) => !s.applicable))
  // starting selection only; gameplay-changing tips are never preselected
  const picked = new SvelteSet<string>(
    untrack(() => report.optimize.filter((s) => s.applicable && !s.gameplay).map((s) => s.id)),
  )
  const flow = new FixFlow()

  async function copy(text: string) {
    try {
      await navigator.clipboard.writeText(text)
      notify(t('optimize.copied'), 'success')
    } catch {
      notify(t('errors.clipboard'), 'error')
    }
  }
</script>

{#if report.optimize.length}
  <ul class="items">
    {#each applicable as s (s.id)}
      <li class="item">
        <div class="item-row top">
          <Check checked={picked.has(s.id)} label={t(`optimize.tips.${s.id}.title`)} onchange={(on) => (on ? picked.add(s.id) : picked.delete(s.id))} />
          <div class="item-text">
            <div class="title-row">
              <b>{t(`optimize.tips.${s.id}.title`)}</b>
              {#if s.gameplay}<span class="tag">{t('optimize.gameplay')}</span>{/if}
            </div>
            <span class="mono">{s.file} · {s.key} {s.current} → {s.recommended}</span>
            <p>{t(`optimize.tips.${s.id}.why`)}</p>
          </div>
        </div>
      </li>
    {/each}

    {#each manual as s (s.id)}
      <li class="item">
        <div class="item-row top">
          <span class="manual"><Icon name="gauge" size={16} stroke={1.8} /></span>
          <div class="item-text">
            <b>{t(`optimize.tips.${s.id}.title`)}</b>
            <span class="mono">{s.file}</span>
            <p>{t(`optimize.tips.${s.id}.why`)}</p>
            <button class="btn small copy" onclick={() => copy(s.recommended)}>
              <Icon name="copy" size={14} stroke={2} />
              {t('optimize.copy')}
            </button>
          </div>
        </div>
      </li>
    {/each}
  </ul>
  <p class="hint">{t('optimize.hint')}</p>

  {#if applicable.length}
    <ActionBar text={t('optimize.selected', { n: picked.size })}>
      <button class="btn primary" onclick={() => flow.start({ kind: 'optimize', ids: [...picked] })} disabled={!picked.size || flow.busy}>
        {t('optimize.apply')}
      </button>
    </ActionBar>
  {/if}
{:else}
  <div class="empty">
    <span class="empty-icon"><Icon name="check" size={28} stroke={2} /></span>
    <b>{t('optimize.cleanTitle')}</b>
    <p>{t('optimize.cleanText')}</p>
  </div>
{/if}

{#if flow.open}
  <PlanDialog
    title={t('optimize.planTitle')}
    previews={flow.previews}
    busy={flow.busy}
    onconfirm={() => flow.confirm()}
    oncancel={() => flow.cancel()}
  />
{/if}

<style>
  .top {
    align-items: flex-start;
  }
  .title-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .manual {
    width: 22px;
    height: 22px;
    display: grid;
    place-items: center;
    color: var(--text-3);
    flex: none;
  }
  .copy {
    margin-top: 12px;
  }
</style>
