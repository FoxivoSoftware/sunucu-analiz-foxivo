<script lang="ts">
  import { t } from '$lib/i18n.svelte'
  import { settings } from '$lib/settings.svelte'
  import { FixFlow } from '$lib/fixflow.svelte'
  import type { Report } from '$lib/api'
  import Icon from '$lib/components/Icon.svelte'
  import ActionBar from '$lib/components/ActionBar.svelte'
  import PlanDialog from '$lib/components/PlanDialog.svelte'

  let { report }: { report: Report } = $props()

  // start from what the user typed for a previous server
  let values = $state<Record<string, string>>({ ...settings.replaceValues })
  const flow = new FixFlow()

  const filled = $derived(report.placeholders.filter((h) => values[h.value]?.trim()).length)

  function preview() {
    const picked = Object.fromEntries(
      report.placeholders.map((h) => [h.value, values[h.value]?.trim() ?? '']).filter(([, v]) => v),
    )
    settings.replaceValues = { ...settings.replaceValues, ...picked }
    flow.start({ kind: 'replace', values: picked })
  }
</script>

{#if report.placeholders.length}
  <ul class="items">
    {#each report.placeholders as h (h.value)}
      <li class="item">
        <div class="item-row">
          <div class="item-text">
            <b class="value">{h.value}{#if h.detected}<span class="tag found">{t('replace.detected')}</span>{/if}</b>
            <span class="mono">{t('replace.spots', { n: h.count, files: h.files.length })}</span>
          </div>
          <Icon name="arrowRight" size={16} stroke={2} />
          <input
            class="field"
            bind:value={values[h.value]}
            placeholder={t('replace.placeholder')}
            aria-label={t('replace.newValue', { value: h.value })}
            spellcheck="false"
          />
        </div>
      </li>
    {/each}
  </ul>
  <p class="hint">{t('replace.hint')}</p>

  <ActionBar text={t('replace.filled', { n: filled, total: report.placeholders.length })}>
    <button class="btn primary" onclick={preview} disabled={!filled || flow.busy}>{t('replace.preview')}</button>
  </ActionBar>
{:else}
  <div class="empty">
    <span class="empty-icon"><Icon name="check" size={28} stroke={2} /></span>
    <b>{t('replace.cleanTitle')}</b>
    <p>{t('replace.cleanText')}</p>
  </div>
{/if}

{#if flow.open}
  <PlanDialog
    title={t('replace.planTitle')}
    previews={flow.previews}
    busy={flow.busy}
    onconfirm={() => flow.confirm()}
    oncancel={() => flow.cancel()}
  />
{/if}

<style>
  .value {
    font-family: ui-monospace, 'Cascadia Code', 'SF Mono', Consolas, monospace;
    font-size: 14px !important;
  }
  .item-row > :global(svg) {
    color: var(--text-3);
    flex: none;
  }
  .found {
    margin-left: 10px;
    vertical-align: 2px;
    font-family: var(--font);
    color: #c4b5fd;
    background: rgba(167, 139, 250, 0.14);
  }
  .field {
    width: 46%;
    height: 40px;
    padding: 0 14px;
    border-radius: 12px;
    font: inherit;
    font-size: 14px;
    color: var(--text);
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.12);
    outline: none;
    transition: border-color 0.15s, box-shadow 0.15s;
  }
  .field::placeholder {
    color: #75757e;
  }
  .field:focus {
    border-color: rgba(165, 180, 252, 0.6);
    box-shadow: 0 0 0 4px rgba(129, 140, 248, 0.12);
  }
</style>
