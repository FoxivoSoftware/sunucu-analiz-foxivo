<script lang="ts">
  import { onMount } from 'svelte'
  import { fade } from 'svelte/transition'
  import { settings, firstRun } from '$lib/settings.svelte'
  import { nav } from '$lib/nav.svelte'
  import { installerLanguage, inTauri } from '$lib/native'
  import { cleanupBackups } from '$lib/api'
  import { issueCount } from '$lib/report'
  import Glow, { type Mood } from '$lib/components/Glow.svelte'
  import TitleBar from '$lib/components/TitleBar.svelte'
  import PoweredBy from '$lib/components/PoweredBy.svelte'
  import HelpAssistant from '$lib/components/HelpAssistant.svelte'
  import Toast from '$lib/components/Toast.svelte'
  import UpdateBanner from '$lib/components/UpdateBanner.svelte'
  import { checkUpdates } from '$lib/updates.svelte'
  import { t } from '$lib/i18n.svelte'
  import { invoke } from '@tauri-apps/api/core'
  import Home from './screens/Home.svelte'
  import Result from './screens/Result.svelte'
  import Module from './screens/Module.svelte'
  import Settings from './screens/Settings.svelte'

  // layout is designed on a 1360x860 canvas; scale it to the window so a small
  // window doesn't feel cramped and a maximized one doesn't feel empty
  const DESIGN_W = 1360
  const DESIGN_H = 860
  const SCALE_STEP = { compact: 0.9, auto: 1, large: 1.12 }

  let viewport = $state({ w: window.innerWidth, h: window.innerHeight })

  $effect(() => {
    const fit = Math.min(viewport.w / DESIGN_W, viewport.h / DESIGN_H)
    const zoom = Math.min(1.35, Math.max(0.72, fit * SCALE_STEP[settings.uiScale]))
    document.documentElement.style.zoom = zoom.toFixed(3)
  })

  const mood = $derived.by((): Mood => {
    if (nav.screen === 'settings') return 'calm'
    if (nav.screen === 'home' || !nav.report) return 'home'
    if (nav.report.score < 40) return 'critical'
    return issueCount(nav.report) ? 'warn' : 'clean'
  })

  if (firstRun) {
    installerLanguage().then((lang) => {
      if (lang) settings.lang = lang
    })
  }

  $effect(() => {
    document.documentElement.classList.toggle('reduce-motion', settings.reduceMotion)
  })

  // the tray lives in Rust; keep its menu language and close behaviour in sync
  $effect(() => {
    if (!inTauri) return
    invoke('set_tray_labels', { show: t('tray.show'), quit: t('tray.quit') }).catch(() => {})
  })
  $effect(() => {
    if (!inTauri) return
    invoke('set_close_to_tray', { enabled: settings.closeToTray }).catch(() => {})
  })

  onMount(() => {
    cleanupBackups().catch(() => {})
    if (settings.autoUpdate) setTimeout(() => checkUpdates(true), 4000)
    const resize = () => (viewport = { w: window.innerWidth, h: window.innerHeight })
    window.addEventListener('resize', resize)
    return () => window.removeEventListener('resize', resize)
  })
</script>

<Glow {mood} animate={settings.glow && !settings.reduceMotion} />
<TitleBar />

{#key nav.screen}
  <main in:fade={{ duration: 200, delay: 40 }}>
    {#if nav.screen === 'home'}
      <Home />
    {:else if nav.screen === 'result' && nav.report}
      <Result report={nav.report} />
    {:else if nav.screen === 'module' && nav.report && nav.module}
      <Module report={nav.report} id={nav.module} />
    {:else}
      <Settings />
    {/if}
  </main>
{/key}

<PoweredBy />
<HelpAssistant />
<Toast />
<UpdateBanner />

<style>
  main {
    position: fixed;
    inset: 0;
  }
</style>
