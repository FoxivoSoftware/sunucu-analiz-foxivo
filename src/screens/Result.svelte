<script lang="ts">
  import { fade, slide } from 'svelte/transition'
  import { t } from '$lib/i18n.svelte'
  import { openModule, go, nav } from '$lib/nav.svelte'
  import { cards, issueCount, nextStep, type ModuleId } from '$lib/report'
  import ScoreRing from '$lib/components/ScoreRing.svelte'
  import { shareReport } from '$lib/share'
  import type { Report } from '$lib/api'
  import Icon from '$lib/components/Icon.svelte'

  let { report }: { report: Report } = $props()

  let showHelp = $state(false)
  const gain = $derived(report.breakdown.reduce((n, p) => n + p.points, 0))

  const list = $derived(cards(report))
  const issues = $derived(issueCount(report))
  const next = $derived(nextStep(report))
  const accent = $derived(
    report.score < 40
      ? 'linear-gradient(90deg, #ffb4a8 0%, #ff8fa3 55%, #f9a8d4 100%)'
      : issues
        ? 'linear-gradient(90deg, #fed7aa 0%, #fdba74 50%, #f9a8d4 100%)'
        : 'linear-gradient(90deg, #bbf7d0 0%, #86efac 50%, #a7f3d0 100%)',
  )
</script>

<div class="result">
  <div class="ctx glass" title={report.root}>
    <i class="dot" class:ok={!issues} aria-hidden="true"></i>
    <b>{report.name}</b>
    <span>· {report.platform}{report.mcVersion ? ` ${report.mcVersion}` : ''} · {t('result.plugins', { n: report.plugins.length })}</span>
  </div>

  <div class="hero-wrap">
  <div class="hero">
    <ScoreRing score={report.score} />
    <div class="heading">
      <h1>
        {#if issues}
          {t('result.title', { n: issues })}<br />
          <span class="gradient-text" style:background-image={accent}>{t('result.titleAccent')}</span>
        {:else}
          {t('result.cleanTitle')}<br />
          <span class="gradient-text" style:background-image={accent}>{t('result.cleanAccent')}</span>
        {/if}
      </h1>
      <p class="sub">{t('result.subtitle')}</p>
      {#if report.breakdown.length}
        <button class="raise" onclick={() => (showHelp = !showHelp)} aria-expanded={showHelp}>
          <Icon name="gauge" size={15} stroke={2} />
          {t('result.raise', { n: gain })}
        </button>
      {/if}
    </div>
  </div>

  {#if showHelp}
    <div class="help glass" transition:slide={{ duration: 240 }}>
      <p class="help-title">{t('result.raiseTitle', { score: report.score, max: report.score + gain })}</p>
      <ul>
        {#each report.breakdown as p (p.id)}
          <li>
            <span class="plus">+{p.points}</span>
            <span class="what">{t(`result.raiseTip.${p.id}`)}</span>
            <button class="btn small" onclick={() => openModule(p.id as ModuleId)}>{t('result.fix')}</button>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
  </div>

  {#if report.running}
    <div class="warn">
      <Icon name="alert" size={17} stroke={2} />
      <span>{t('result.running')}</span>
    </div>
  {:else if next}
    <button class="next glass" onclick={() => openModule(next)} in:fade|global={{ duration: 420, delay: 120 }}>
      <span class="step">{t('result.nextLabel')}</span>
      <b>{t(`result.next.${next}`)}</b>
      <span class="go">{t('result.start')} <Icon name="arrowRight" size={16} stroke={2} /></span>
    </button>
  {/if}

  <div class="tiles">
    {#each list as c, i (c.id)}
      <button
        class="tile glass"
        class:hot={c.id === next}
        onclick={() => openModule(c.id)}
        in:fade|global={{ duration: 420, delay: 220 + i * 70 }}
      >
        <div class="row">
          <Icon name={c.icon} size={24} stroke={1.6} />
          <span class="badge {c.tone}"><i></i>{c.badge}</span>
        </div>
        <h3>{t(`modules.${c.id}`)}</h3>
        <p>{c.hint}</p>
      </button>
    {/each}
  </div>
</div>

<div class="dock glass">
  <div class="state">
    <Icon name="lock" size={18} stroke={2} />
    <span><b>{t('result.readonlyTitle')}</b> · {t('result.readonly')}</span>
  </div>
  <div class="actions">
    <button class="btn" onclick={() => go('home')}>{t('result.another')}</button>
    <button class="btn primary" onclick={() => nav.report && shareReport(nav.report)}>
      <Icon name="download" size={16} stroke={2} />
      {t('result.share')}
    </button>
  </div>
</div>

<style>
  .result {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: calc(var(--bar-h) + 30px) 24px 0;
  }

  .ctx {
    height: 36px;
    max-width: min(560px, 100%);
    padding: 0 16px 0 14px;
    border-radius: 999px;
    display: flex;
    align-items: center;
    gap: 10px;
    border: 1px solid rgba(255, 255, 255, 0.11);
    font-size: 13.5px;
    color: #c7c7cc;
    white-space: nowrap;
  }
  .ctx b {
    color: var(--text);
    font-weight: 600;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .ctx span {
    margin-left: -5px;
    flex: none;
  }

  /* live status dot: solid core with a ring that keeps expanding */
  .dot {
    --c: 255, 178, 107;
    position: relative;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: rgb(var(--c));
    flex: none;
    animation: glow 2s ease-in-out infinite;
  }
  .dot.ok {
    --c: 127, 224, 176;
  }
  .dot::after {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: rgb(var(--c));
    animation: ripple 2s cubic-bezier(0, 0, 0.2, 1) infinite;
  }
  @keyframes glow {
    0%, 100% { box-shadow: 0 0 5px rgba(var(--c), 0.5); }
    50% { box-shadow: 0 0 11px rgba(var(--c), 0.9); }
  }
  @keyframes ripple {
    from { transform: scale(1); opacity: 0.5; }
    to { transform: scale(2.6); opacity: 0; }
  }

  .hero-wrap {
    position: relative;
    margin-top: 18px;
  }
  .hero {
    display: flex;
    align-items: center;
    gap: 30px;
  }
  h1 {
    font-size: 40px;
    line-height: 1.12;
    font-weight: 600;
    letter-spacing: -0.034em;
  }
  .sub {
    margin-top: 8px;
    font-size: 14.5px;
    color: var(--text-2);
  }
  .raise {
    margin-top: 12px;
    height: 32px;
    padding: 0 14px 0 11px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-size: 13px;
    font-weight: 600;
    color: #e9e3ff;
    background: rgba(167, 139, 250, 0.12);
    border: 1px solid rgba(167, 139, 250, 0.3);
    transition: background 0.15s;
  }
  .raise:hover,
  .raise[aria-expanded='true'] {
    background: rgba(167, 139, 250, 0.22);
  }
  .raise :global(svg) {
    color: #c4b5fd;
  }

  /* floats over the cards so opening it doesn't push the page down */
  .help {
    position: absolute;
    top: calc(100% + 10px);
    left: 50%;
    translate: -50% 0;
    width: min(560px, 90vw);
    z-index: 12;
    padding: 16px 16px 10px;
    border-radius: 20px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(26, 26, 32, 0.94);
    box-shadow: 0 30px 70px rgba(0, 0, 0, 0.55);
  }
  .help-title {
    padding: 0 4px 10px;
    font-size: 14px;
    font-weight: 600;
  }
  .help ul {
    list-style: none;
  }
  .help li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 9px 4px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }
  .plus {
    min-width: 46px;
    height: 26px;
    border-radius: 99px;
    display: grid;
    place-items: center;
    font-size: 13px;
    font-weight: 700;
    color: var(--green);
    background: rgba(111, 211, 154, 0.13);
    font-variant-numeric: tabular-nums;
  }
  .what {
    flex: 1;
    font-size: 13.5px;
    color: #d6d6dc;
  }

  .next {
    margin-top: 22px;
    width: min(900px, 100%);
    height: 60px;
    padding: 0 10px 0 22px;
    border-radius: 20px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    display: flex;
    align-items: center;
    gap: 14px;
    text-align: left;
    transition: border-color 0.2s, transform 0.2s var(--ease);
  }
  .next:hover {
    border-color: rgba(255, 255, 255, 0.24);
  }
  .step {
    font-size: 11.5px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #c4b5fd;
    flex: none;
  }
  .next b {
    flex: 1;
    min-width: 0;
    font-size: 15px;
    font-weight: 600;
  }
  .go {
    height: 40px;
    padding: 0 16px 0 18px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    font-weight: 600;
    color: #0a0a0d;
    background: var(--text);
    flex: none;
  }

  .warn {
    margin-top: 22px;
    width: min(900px, 100%);
    padding: 14px 18px;
    border-radius: 18px;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 14px;
    color: #ffe1c4;
    background: rgba(255, 184, 113, 0.1);
    border: 1px solid rgba(255, 184, 113, 0.3);
  }
  .warn :global(svg) {
    color: var(--amber);
    flex: none;
  }

  .tiles {
    margin-top: 16px;
    width: min(900px, 100%);
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }
  .tile {
    height: 124px;
    padding: 18px 18px 16px;
    border-radius: 22px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    flex-direction: column;
    text-align: left;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 16px 40px rgba(0, 0, 0, 0.25);
    transition: border-color 0.2s, transform 0.2s var(--ease), background 0.2s;
  }
  .tile:hover {
    border-color: rgba(255, 255, 255, 0.2);
    background: rgba(36, 36, 44, 0.62);
  }
  .tile.hot {
    border-color: rgba(196, 181, 253, 0.35);
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .row :global(svg) {
    color: #e5e5ea;
  }
  .tile h3 {
    margin-top: auto;
    font-size: 15.5px;
    font-weight: 600;
    letter-spacing: -0.015em;
  }
  .tile p {
    margin-top: 4px;
    font-size: 13px;
    color: #9a9aa2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .badge {
    font-size: 12px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
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

  .dock {
    position: fixed;
    left: 50%;
    bottom: 72px;
    transform: translateX(-50%);
    width: min(660px, calc(100vw - 48px));
    height: 60px;
    border-radius: 30px;
    padding: 0 10px 0 22px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border: 1px solid rgba(255, 255, 255, 0.14);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.1), 0 24px 70px rgba(0, 0, 0, 0.5);
  }
  .state {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    white-space: nowrap;
    font-size: 13.5px;
    color: #b6b6bd;
  }
  .state span {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .state :global(svg) {
    color: var(--green);
  }
  .state b {
    color: var(--text);
    font-weight: 600;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
</style>
