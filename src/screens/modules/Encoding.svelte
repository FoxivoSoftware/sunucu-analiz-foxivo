<script lang="ts">
  import { SvelteSet } from 'svelte/reactivity'
  import { t } from '$lib/i18n.svelte'
  import { FixFlow } from '$lib/fixflow.svelte'
  import type { Report } from '$lib/api'
  import Icon from '$lib/components/Icon.svelte'
  import Check from '$lib/components/Check.svelte'
  import ActionBar from '$lib/components/ActionBar.svelte'
  import PlanDialog from '$lib/components/PlanDialog.svelte'

  let { report }: { report: Report } = $props()

  const skipped = new SvelteSet<string>()
  const flow = new FixFlow()
  const selected = $derived(report.encoding.filter((e) => !skipped.has(e.file)).map((e) => e.file))

  const set = (file: string, on: boolean) => (on ? skipped.delete(file) : skipped.add(file))
</script>

{#if report.encoding.length}
  <ul class="items">
    {#each report.encoding as e (e.file)}
      <li class="item">
        <div class="item-row">
          <Check checked={!skipped.has(e.file)} label={e.file} onchange={(on) => set(e.file, on)} />
          <div class="item-text">
            <b class="mono-title">{e.file}</b>
            <span class="mono">{t(`encoding.kind.${e.kind}`)} · {t('encoding.lines', { n: e.lines })}</span>
          </div>
        </div>
        <div class="diff">
          <span class="old">{e.before}</span>
          <span class="new">{e.after}</span>
        </div>
      </li>
    {/each}
  </ul>
  <p class="hint">{t('encoding.hint')}</p>

  <ActionBar text={t('encoding.selected', { n: selected.length })}>
    <button class="btn primary" onclick={() => flow.start({ kind: 'encoding', files: selected })} disabled={!selected.length || flow.busy}>
      {t('encoding.fix')}
    </button>
  </ActionBar>
{:else}
  <div class="empty">
    <span class="empty-icon"><Icon name="check" size={28} stroke={2} /></span>
    <b>{t('encoding.cleanTitle')}</b>
    <p>{t('encoding.cleanText')}</p>
  </div>
{/if}

{#if flow.open}
  <PlanDialog
    title={t('encoding.planTitle')}
    previews={flow.previews}
    busy={flow.busy}
    onconfirm={() => flow.confirm()}
    oncancel={() => flow.cancel()}
  />
{/if}

<style>
  .mono-title {
    font-family: ui-monospace, 'Cascadia Code', 'SF Mono', Consolas, monospace;
    font-size: 13.5px !important;
    overflow-wrap: anywhere;
  }
</style>
