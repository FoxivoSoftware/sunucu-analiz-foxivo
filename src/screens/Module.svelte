<script lang="ts">
  import { t } from '$lib/i18n.svelte'
  import { back } from '$lib/nav.svelte'
  import { cards, type ModuleId } from '$lib/report'
  import type { Report } from '$lib/api'
  import Icon from '$lib/components/Icon.svelte'
  import IconTile from '$lib/components/IconTile.svelte'
  import Security from './modules/Security.svelte'
  import Replace from './modules/Replace.svelte'
  import Encoding from './modules/Encoding.svelte'
  import Optimize from './modules/Optimize.svelte'
  import Anticheat from './modules/Anticheat.svelte'
  import Backups from './modules/Backups.svelte'

  let { report, id }: { report: Report; id: ModuleId } = $props()

  const card = $derived(cards(report).find((c) => c.id === id))
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && !document.querySelector('[role=dialog]') && back()} />

<div class="page">
  <div class="col">
    <button class="crumb" onclick={back}>
      <Icon name="chevronLeft" size={16} stroke={2} />
      {t('module.back')}
    </button>

    <div class="head">
      {#if card}
        <IconTile icon={card.icon} tone={card.tone} />
      {/if}
      <div class="titles">
        <h1>{t(`modules.${id}`)}</h1>
        <p>{t(`${id}.lead`, { n: report.plugins.length })}</p>
      </div>
      {#if card}
        <span class="badge {card.tone}"><i></i>{card.badge}</span>
      {/if}
    </div>

    <div class="body">
      {#if id === 'security'}
        <Security {report} />
      {:else if id === 'replace'}
        <Replace {report} />
      {:else if id === 'encoding'}
        <Encoding {report} />
      {:else if id === 'optimize'}
        <Optimize {report} />
      {:else if id === 'anticheat'}
        <Anticheat {report} />
      {:else}
        <Backups />
      {/if}
    </div>
  </div>
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
    overflow-y: auto;
    padding: calc(var(--bar-h) + 22px) 24px 180px;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.12) transparent;
  }
  .col {
    width: min(800px, 100%);
    margin: 0 auto;
  }
  .crumb {
    height: 32px;
    padding: 0 12px 0 8px;
    margin-left: -8px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 13.5px;
    font-weight: 500;
    color: var(--text-2);
    transition: background 0.15s, color 0.15s;
  }
  .crumb:hover {
    background: rgba(255, 255, 255, 0.06);
    color: var(--text);
  }
  .head {
    margin-top: 12px;
    display: flex;
    align-items: flex-start;
    gap: 18px;
  }
  .tile {
    width: 56px;
    height: 56px;
    border-radius: 18px;
    display: grid;
    place-items: center;
    color: #e5e5ea;
    border: 1px solid rgba(255, 255, 255, 0.1);
    flex: none;
  }
  .titles {
    flex: 1;
    min-width: 0;
  }
  h1 {
    font-size: 30px;
    line-height: 1.15;
    font-weight: 600;
    letter-spacing: -0.03em;
  }
  .titles p {
    margin-top: 8px;
    font-size: 15px;
    line-height: 1.55;
    color: var(--text-2);
    max-width: 580px;
  }
  .badge {
    margin-top: 6px;
    font-size: 12px;
    font-weight: 600;
    padding: 5px 11px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex: none;
  }
  .badge i {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }
  .red { color: var(--red); background: rgba(255, 122, 112, 0.15); }
  .amber { color: var(--amber); background: rgba(255, 184, 113, 0.15); }
  .green { color: var(--green); background: rgba(111, 211, 154, 0.14); }
  .neutral { color: #c7c7cc; background: rgba(255, 255, 255, 0.08); }
  .body {
    margin-top: 20px;
  }
</style>
