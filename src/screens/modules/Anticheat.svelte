<script lang="ts">
  import { t } from '$lib/i18n.svelte'
  import { openLink } from '$lib/native'
  import { copyPrompt } from '$lib/ai'
  import type { Report } from '$lib/api'
  import Icon from '$lib/components/Icon.svelte'

  let { report }: { report: Report } = $props()

  const GRIM_URL = 'https://modrinth.com/plugin/grimac'

  const many = $derived(report.anticheats.length > 1)
  const server = $derived(`${report.platform}${report.mcVersion ? ` ${report.mcVersion}` : ''}`)
  const names = $derived(report.anticheats.map((a) => `${a.name}${a.version ? ` ${a.version}` : ''}`).join(', '))

  const askMany = () => copyPrompt(t('anticheat.ai.many', { server, names, plugins: report.plugins.length }))
  const askChoose = () => copyPrompt(t('anticheat.ai.choose', { server, plugins: report.plugins.map((p) => p.name).join(', ') }))
</script>

{#if report.anticheats.length}
  {#if many}
    <div class="item notice">
      <div class="item-row top">
        <span class="sev medium"><i></i>{t('sev.medium')}</span>
        <div class="item-text">
          <b>{t('anticheat.manyTitle', { names: report.anticheats.map((a) => a.name).join(', ') })}</b>
          <p>{t('anticheat.manyWhy')}</p>
          <div class="actions">
            <button class="btn small ai" onclick={askMany}>
              <Icon name="copy" size={13} stroke={2} />
              {t('security.ai.button')}
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <p class="section-title">{t('anticheat.found')}</p>
  <ul class="items">
    {#each report.anticheats as a (a.file)}
      <li class="item">
        <div class="item-row">
          <span class="ico"><Icon name="shieldAlert" size={18} stroke={1.7} /></span>
          <div class="item-text">
            <b>{a.name}{a.version ? ` ${a.version}` : ''}</b>
            <span class="mono">plugins/{a.file}</span>
          </div>
        </div>
      </li>
    {/each}
  </ul>

  <div class="item tip">
    <div class="item-text">
      <b>{t('anticheat.tipTitle')}</b>
      <p>{t('anticheat.tip')}</p>
    </div>
  </div>
{:else}
  <div class="item notice neutral">
    <div class="item-row top">
      <span class="ico"><Icon name="shieldAlert" size={18} stroke={1.7} /></span>
      <div class="item-text">
        <b>{t('anticheat.noneTitle')}</b>
        <p>{t('anticheat.noneText')}</p>
      </div>
    </div>
  </div>

  <p class="section-title">{t('anticheat.suggestTitle')}</p>
  <div class="item pick">
    <div class="item-row top">
      <span class="ico rec"><Icon name="shieldCheck" size={18} stroke={1.8} /></span>
      <div class="item-text">
        <div class="name-row">
          <b>GrimAC</b>
          <span class="tag free">{t('anticheat.free')}</span>
        </div>
        <p>{t('anticheat.grim')}</p>
        <div class="actions">
          <button class="btn small" onclick={() => openLink(GRIM_URL)}>
            <Icon name="external" size={13} stroke={2} />
            {t('anticheat.openPage')}
          </button>
          <button class="btn small ai" onclick={askChoose}>
            <Icon name="copy" size={13} stroke={2} />
            {t('security.ai.button')}
          </button>
        </div>
      </div>
    </div>
  </div>
  <p class="hint">{t('anticheat.others')}</p>
{/if}
<p class="hint">{t('anticheat.disclaimer')}</p>

<style>
  .top {
    align-items: flex-start;
  }
  .notice {
    border-color: rgba(255, 184, 113, 0.28);
  }
  .notice.neutral {
    border-color: rgba(255, 255, 255, 0.1);
  }
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
  .ico.rec {
    color: var(--green);
    background: rgba(111, 211, 154, 0.12);
  }
  .pick {
    border-color: rgba(111, 211, 154, 0.25);
  }
  .name-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .tag.free {
    color: var(--green);
    background: rgba(111, 211, 154, 0.12);
  }
  .actions {
    margin-top: 12px;
    display: flex;
    gap: 8px;
  }
  .ai {
    border-color: rgba(167, 139, 250, 0.35);
    color: #ddd6fe;
  }
  .ai:hover {
    background: rgba(167, 139, 250, 0.14);
  }
  .tip {
    margin-top: 10px;
  }
</style>
