<script lang="ts">
  import { onMount } from 'svelte'
  import { fade } from 'svelte/transition'
  import { t } from '$lib/i18n.svelte'
  import { nav, startScan } from '$lib/nav.svelte'
  import { settings } from '$lib/settings.svelte'
  import { pickFolder, onFileDrop, folderName, inTauri } from '$lib/native'
  import Icon from '$lib/components/Icon.svelte'
  import logo from '$lib/assets/logo.png'

  let dragging = $state(false)

  // plugins are the slow part, so they get most of the bar
  const percent = $derived.by(() => {
    const p = nav.progress
    if (!p) return 4
    const frac = p.total ? p.done / p.total : 0
    if (p.phase === 'plugins') return 6 + frac * 54
    if (p.phase === 'verify') return 64
    if (p.phase === 'files') return 72 + frac * 26
    return 100
  })

  const phaseLabel = $derived(
    nav.progress && nav.progress.phase !== 'done'
      ? t(`home.phase.${nav.progress.phase}`, { done: nav.progress.done + 1, total: nav.progress.total })
      : t('home.scanning'),
  )

  const steps = [
    { n: 1, key: 'pick' },
    { n: 2, key: 'review' },
    { n: 3, key: 'fix' },
  ]

  async function browse() {
    if (nav.scanning) return
    if (!inTauri) return startScan('demo') // browser preview uses the sample report
    const folder = await pickFolder(t('home.pickTitle'))
    if (folder) startScan(folder)
  }

  onMount(() => {
    const off = onFileDrop((e) => {
      if (nav.scanning) return
      if (e.type === 'enter' || e.type === 'over') dragging = true
      else if (e.type === 'leave') dragging = false
      else {
        dragging = false
        if (e.paths[0]) startScan(e.paths[0])
      }
    })
    return () => {
      off.then((fn) => fn())
    }
  })
</script>

<div class="home">
  <img class="mark" src={logo} alt="" draggable="false" />
  <h1>{t('home.title')} <span class="gradient-text">{t('home.titleAccent')}</span></h1>
  <p class="sub">{t('home.subtitle')}</p>

  <div class="zone glass" class:dragging class:busy={nav.scanning}>
    {#if nav.scanning}
      <div class="state scan" in:fade={{ duration: 160 }}>
        <span class="loader" aria-hidden="true"></span>
        <b>{phaseLabel}</b>
        <div class="track" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(percent)}>
          <i style:width="{percent}%"></i>
        </div>
        <span class="current">{nav.progress?.current || t('home.scanningHint')}</span>
      </div>
    {:else}
      <div class="state" in:fade={{ duration: 160 }}>
        <span class="folder"><Icon name={dragging ? 'download' : 'folderOpen'} size={30} stroke={1.5} /></span>
        <b>{dragging ? t('home.dropActive') : t('home.drop')}</b>
        <span>{t('home.dropHint')}</span>
        <button class="btn primary big" onclick={browse}>
          <Icon name="folder" size={18} stroke={1.8} />
          {t('home.choose')}
        </button>
      </div>
    {/if}
  </div>

  {#if nav.error}
    <div class="error" role="alert" in:fade>
      <Icon name="alert" size={16} stroke={2} />
      <span>{nav.error}</span>
    </div>
  {:else if settings.lastPath && !nav.scanning && inTauri}
    <button class="recent" onclick={() => settings.lastPath && startScan(settings.lastPath)}>
      <Icon name="history" size={15} stroke={1.8} />
      {t('home.recent', { name: folderName(settings.lastPath) })}
    </button>
  {:else}
    <p class="note"><Icon name="lock" size={13} stroke={2} />{t('home.note')}</p>
  {/if}

  <ol class="steps">
    {#each steps as s (s.n)}
      <li>
        <span class="num">{s.n}</span>
        <div>
          <b>{t(`home.steps.${s.key}`)}</b>
          <span>{t(`home.steps.${s.key}Hint`)}</span>
        </div>
      </li>
    {/each}
  </ol>
</div>

<style>
  .home {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--bar-h) 24px 70px;
  }

  .mark {
    width: 76px;
    height: 76px;
    border-radius: 22px;
    box-shadow:
      0 0 0 1px rgba(255, 255, 255, 0.1),
      0 18px 40px rgba(0, 0, 0, 0.5),
      0 0 60px rgba(99, 102, 241, 0.18);
    animation: float-in 0.7s var(--ease) both;
  }
  @keyframes float-in {
    from {
      opacity: 0;
      transform: translateY(8px) scale(0.94);
    }
  }

  h1 {
    margin-top: 26px;
    font-size: 44px;
    line-height: 1.12;
    font-weight: 600;
    letter-spacing: -0.034em;
    text-align: center;
  }
  h1 span {
    background-image: linear-gradient(90deg, #c7d2fe 0%, #d8b4fe 55%, #fbcfe8 100%);
  }
  .sub {
    margin-top: 12px;
    font-size: 17px;
    line-height: 1.5;
    color: var(--text-2);
    text-align: center;
  }

  .zone {
    margin-top: 32px;
    width: min(640px, 100%);
    min-height: 236px;
    border-radius: 28px;
    border: 1.5px dashed rgba(255, 255, 255, 0.16);
    display: grid;
    place-items: center;
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.4);
    transition:
      border-color 0.25s,
      background 0.25s,
      transform 0.25s var(--ease);
  }
  .zone.dragging {
    border-color: rgba(165, 180, 252, 0.7);
    background: rgba(99, 102, 241, 0.1);
    transform: scale(1.01);
  }
  .zone.busy {
    border-style: solid;
    border-color: rgba(255, 255, 255, 0.1);
  }

  .state {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 30px 24px;
  }
  .folder {
    width: 60px;
    height: 60px;
    border-radius: 18px;
    display: grid;
    place-items: center;
    color: #dcdcff;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
  }
  .state b {
    margin-top: 16px;
    font-size: 18px;
    font-weight: 600;
    letter-spacing: -0.015em;
  }
  .state > span:not(.folder):not(.loader) {
    margin-top: 6px;
    font-size: 14px;
    color: var(--text-2);
  }
  .big {
    margin-top: 20px;
    height: 46px;
    padding: 0 24px 0 20px;
    font-size: 15px;
  }

  .scan {
    width: min(440px, 100%);
  }
  .track {
    margin-top: 18px;
    width: 100%;
    height: 8px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.08);
    overflow: hidden;
  }
  .track i {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: linear-gradient(90deg, #a5b4fc, #d8b4fe, #f9a8d4, #a5b4fc);
    background-size: 300% 100%;
    animation: flow 2.4s linear infinite;
    transition: width 0.45s var(--ease);
  }
  @keyframes flow {
    to {
      background-position: -300% 0;
    }
  }
  .current {
    margin-top: 10px !important;
    max-width: 100%;
    font-family: ui-monospace, 'Cascadia Code', Consolas, monospace;
    font-size: 12.5px !important;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .loader {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top-color: #c7d2fe;
    animation: spin 0.9s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .note,
  .recent,
  .error {
    margin-top: 16px;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-size: 13px;
  }
  .note {
    color: var(--text-3);
  }
  .recent {
    height: 34px;
    padding: 0 14px;
    border-radius: 999px;
    color: #d6d6dc;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.09);
    transition: background 0.15s;
  }
  .recent:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  .error {
    max-width: min(640px, 100%);
    padding: 10px 14px;
    border-radius: 14px;
    color: #ffd0cb;
    background: rgba(255, 122, 112, 0.1);
    border: 1px solid rgba(255, 122, 112, 0.25);
  }
  .error :global(svg) {
    color: var(--red);
    flex: none;
  }

  .steps {
    margin-top: 34px;
    list-style: none;
    display: flex;
    gap: 14px;
    width: min(780px, 100%);
  }
  .steps li {
    flex: 1;
    display: flex;
    gap: 12px;
    align-items: flex-start;
    padding: 14px 16px;
    border-radius: 18px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.07);
  }
  .num {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-size: 13px;
    font-weight: 700;
    color: #0a0a0d;
    background: #e8e8f0;
    flex: none;
  }
  .steps b {
    display: block;
    font-size: 14px;
    font-weight: 600;
  }
  .steps li span:not(.num) {
    display: block;
    margin-top: 3px;
    font-size: 12.5px;
    line-height: 1.45;
    color: var(--text-3);
  }
</style>
