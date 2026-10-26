<script lang="ts">
  import { slide } from 'svelte/transition'
  import { SvelteSet } from 'svelte/reactivity'
  import { t } from '$lib/i18n.svelte'
  import { settings } from '$lib/settings.svelte'
  import { openLink } from '$lib/native'
  import type { Finding, Report } from '$lib/api'
  import Icon from '$lib/components/Icon.svelte'
  import ProgressRing from '$lib/components/ProgressRing.svelte'
  import { copyPrompt as copy } from '$lib/ai'

  let { report }: { report: Report } = $props()

  const order = ['high', 'medium', 'low', 'info']
  type Group = { file: string; name: string; version: string | null; items: Finding[]; worst: Finding['severity'] }

  function group(list: Finding[]): Group[] {
    const map = new Map<string, Group>()
    for (const f of list) {
      const p = report.plugins.find((x) => x.file === f.file)
      const g = map.get(f.file) ?? { file: f.file, name: p?.name ?? f.plugin, version: p?.version ?? null, items: [], worst: 'info' }
      g.items.push(f)
      if (order.indexOf(f.severity) < order.indexOf(g.worst)) g.worst = f.severity
      map.set(f.file, g)
    }
    return [...map.values()].sort((a, b) => order.indexOf(a.worst) - order.indexOf(b.worst))
  }

  // what deserves attention vs. notes about the author's own official code
  const findings = $derived(group(report.security.filter((f) => !f.official && f.severity !== 'info')))
  const notes = $derived(group(report.security.filter((f) => f.official || f.severity === 'info')))
  const verified = $derived(report.plugins.filter((p) => p.official))
  const updates = $derived(report.plugins.filter((p) => p.official?.latest))

  const open = new SvelteSet<string>()
  const toggle = (k: string) => (open.has(k) ? open.delete(k) : open.add(k))
  let showNotes = $state(false)
  let showPlugins = $state(false)

  // A ready-to-paste prompt so the user can get a second opinion from any AI
  // assistant. Nothing is sent from the app; the decision stays with the user.
  function copyPrompt(g: Group) {
    const lines = g.items.map((f) => `- ${t(`security.rules.${f.rule}.title`)}: ${f.detail}`).join('\n')
    copy(
      t('security.ai.prompt', {
        server: `${report.platform}${report.mcVersion ? ` ${report.mcVersion}` : ''}`,
        plugin: `${g.name}${g.version ? ` ${g.version}` : ''}`,
        file: g.file,
        findings: lines,
      }),
    )
  }

  const searchOfficial = (name: string) => openLink(`https://modrinth.com/plugins?q=${encodeURIComponent(name)}`)
</script>

{#snippet card(g: Group, actions: boolean)}
  <li class="item group">
    <div class="item-row">
      <span class="sev {g.worst}"><i></i>{t(`sev.${g.worst}`)}</span>
      <div class="item-text">
        <b>{g.name}{g.version ? ` ${g.version}` : ''}</b>
        <span class="mono">plugins/{g.file}</span>
      </div>
      {#if actions}
        <button class="btn small" onclick={() => searchOfficial(g.name)} title={t('security.searchHint')}>
          <Icon name="external" size={13} stroke={2} />
          {t('security.search')}
        </button>
        <button class="btn small ai" onclick={() => copyPrompt(g)} title={t('security.ai.hint')}>
          <Icon name="copy" size={13} stroke={2} />
          {t('security.ai.button')}
        </button>
      {:else if g.items[0]?.official}
        <span class="tag ok">{t('security.officialTag')}</span>
      {/if}
    </div>
    <ul class="rules">
      {#each g.items as f (f.rule)}
        {@const k = `${f.file}:${f.rule}`}
        <li>
          <button class="rule" onclick={() => toggle(k)} aria-expanded={open.has(k)}>
            <i class="dot {f.severity}"></i>
            <span class="title">{t(`security.rules.${f.rule}.title`)}</span>
            <span class="detail">{f.detail}</span>
            <Icon name="chevronLeft" size={13} stroke={2} />
          </button>
          {#if open.has(k)}
            <p class="why" transition:slide={{ duration: 180 }}>
              {t(`security.rules.${f.rule}.why`)}{f.official ? ` ${t('security.officialWhy')}` : ''}
            </p>
          {/if}
        </li>
      {/each}
    </ul>
    {#if actions}
      <p class="advice">{t('security.advice')}</p>
    {/if}
  </li>
{/snippet}

{#if verified.length}
  <div class="verified">
    <ProgressRing done={verified.length} total={report.plugins.length} />
    <div>
      <b>{t('security.verifiedTitle', { n: verified.length, total: report.plugins.length })}</b>
      <span>{t('security.verifiedText')}</span>
    </div>
  </div>
{:else if !settings.verifyOnline}
  <p class="hint top">{t('security.verifyOff')}</p>
{/if}

{#if findings.length}
  <ul class="items">
    {#each findings as g (g.file)}
      {@render card(g, true)}
    {/each}
  </ul>
{:else}
  <div class="empty compact">
    <span class="empty-icon"><Icon name="shieldCheck" size={26} stroke={1.6} /></span>
    <b>{t('security.cleanTitle')}</b>
    <p>{t('security.cleanText', { n: report.plugins.length })}</p>
  </div>
{/if}

<p class="hint">{t('security.disclaimer')}</p>

{#if updates.length}
  <p class="section-title">{t('security.updates', { n: updates.length })}</p>
  <ul class="updates">
    {#each updates as p (p.file)}
      {@const url = `${p.official?.url}/versions`}
      <li>
        <b>{p.name}</b>
        <span class="ver">{p.official?.version} <Icon name="arrowRight" size={12} stroke={2} /> <em>{p.official?.latest}</em></span>
        <button class="btn small" onclick={() => openLink(url)}>
          <Icon name="external" size={13} stroke={2} />
          {t('security.download')}
        </button>
      </li>
    {/each}
  </ul>
  <p class="hint">{t('security.updatesHint')}</p>
{/if}

{#if notes.length}
  <button class="section-title toggle" onclick={() => (showNotes = !showNotes)} aria-expanded={showNotes}>
    {t('security.notes', { n: notes.reduce((n, g) => n + g.items.length, 0) })}
    <Icon name="chevronLeft" size={13} stroke={2} />
  </button>
  {#if showNotes}
    <ul class="items" transition:slide={{ duration: 220 }}>
      {#each notes as g (g.file)}
        {@render card(g, false)}
      {/each}
    </ul>
  {/if}
{/if}

<button class="section-title toggle" onclick={() => (showPlugins = !showPlugins)} aria-expanded={showPlugins}>
  {t('security.pluginList', { n: report.plugins.length })}
  <Icon name="chevronLeft" size={13} stroke={2} />
</button>
{#if showPlugins}
  <ul class="plugins" transition:slide={{ duration: 220 }}>
    {#each report.plugins as p (p.file)}
      <li>
        <div>
          <b>{p.name}</b>
          <span>{p.version ?? ''}</span>
        </div>
        {#if p.official}
          {@const url = p.official.url}
          <button class="status ok" onclick={() => openLink(url)} title={url}>
            <Icon name="check" size={13} stroke={2.6} />
            {t('security.official', { source: p.official.source })}
          </button>
        {:else}
          <span class="status" title={t('security.unverifiedWhy')}>{t('security.unverified')}</span>
        {/if}
      </li>
    {/each}
  </ul>
  <p class="hint">{t('security.unverifiedWhy')}</p>
{/if}

<style>
  .verified {
    margin-bottom: 12px;
    padding: 12px 18px 12px 12px;
    border-radius: 16px;
    display: flex;
    align-items: center;
    gap: 12px;
    background: rgba(111, 211, 154, 0.08);
    border: 1px solid rgba(111, 211, 154, 0.22);
  }
  .verified b {
    display: block;
    font-size: 14px;
    font-weight: 600;
    color: #d9f5e7;
  }
  .verified span {
    display: block;
    margin-top: 2px;
    font-size: 12.5px;
    line-height: 1.5;
    color: #a9cdbb;
  }
  .hint.top {
    margin: 0 0 12px;
  }

  .group {
    padding: 14px 16px;
  }
  .group .item-row {
    gap: 10px;
  }
  .ai {
    border-color: rgba(167, 139, 250, 0.35);
    color: #ddd6fe;
  }
  .ai:hover {
    background: rgba(167, 139, 250, 0.14);
  }
  .tag.ok {
    color: var(--green);
    background: rgba(111, 211, 154, 0.12);
  }

  .rules {
    list-style: none;
    margin-top: 10px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }
  .rule {
    width: 100%;
    min-height: 36px;
    padding: 6px 4px;
    display: flex;
    align-items: center;
    gap: 10px;
    text-align: left;
    border-radius: 8px;
    font-size: 13.5px;
    transition: background 0.15s;
  }
  .rule:hover {
    background: rgba(255, 255, 255, 0.04);
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex: none;
  }
  .dot.high { background: var(--red); }
  .dot.medium { background: var(--amber); }
  .dot.low { background: var(--blue); }
  .dot.info { background: var(--green); }
  .title {
    font-weight: 600;
    color: #e8e8ed;
    flex: none;
  }
  .detail {
    flex: 1;
    min-width: 0;
    font-family: ui-monospace, 'Cascadia Code', Consolas, monospace;
    font-size: 12px;
    color: var(--text-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .rule :global(svg) {
    color: var(--text-3);
    transform: rotate(-90deg);
    transition: transform 0.2s;
    flex: none;
  }
  .rule[aria-expanded='true'] :global(svg) {
    transform: rotate(90deg);
  }
  .why {
    margin: 2px 0 8px 21px;
    font-size: 13px;
    line-height: 1.55;
    color: #c3c3ca;
  }
  .advice {
    margin-top: 8px;
    padding: 9px 12px;
    border-radius: 10px;
    font-size: 12.5px;
    line-height: 1.5;
    color: #cfcfe0;
    background: rgba(255, 255, 255, 0.04);
  }

  .updates {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .updates li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 8px 8px 16px;
    border-radius: 14px;
    background: rgba(141, 180, 255, 0.05);
    border: 1px solid rgba(141, 180, 255, 0.14);
  }
  .updates b {
    flex: 1;
    min-width: 0;
    font-size: 14px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .ver {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-family: ui-monospace, 'Cascadia Code', Consolas, monospace;
    font-size: 12.5px;
    color: var(--text-3);
  }
  .ver em {
    font-style: normal;
    color: var(--blue);
    font-weight: 600;
  }

  .empty.compact {
    padding: 28px 24px;
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .toggle :global(svg) {
    transform: rotate(-90deg);
    transition: transform 0.2s;
  }
  .toggle[aria-expanded='true'] :global(svg) {
    transform: rotate(90deg);
  }

  .plugins {
    list-style: none;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .plugins li {
    padding: 9px 10px 9px 14px;
    border-radius: 14px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.07);
  }
  .plugins li div {
    min-width: 0;
  }
  .plugins b {
    display: block;
    font-size: 13.5px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .plugins span:not(.status) {
    font-size: 12px;
    color: var(--text-3);
  }
  .status {
    height: 26px;
    padding: 0 10px;
    border-radius: 99px;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-2);
    background: rgba(255, 255, 255, 0.06);
    flex: none;
  }
  .status.ok {
    color: var(--green);
    background: rgba(111, 211, 154, 0.12);
  }
  .status.ok:hover {
    background: rgba(111, 211, 154, 0.2);
  }
</style>
