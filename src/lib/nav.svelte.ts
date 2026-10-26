import { scanServer, friendlyError, type Progress, type Report } from './api'
import { settings } from './settings.svelte'
import { inTauri, onScanProgress } from './native'
import type { ModuleId } from './report'

export type Screen = 'home' | 'result' | 'module' | 'settings'

export const nav = $state<{
  screen: Screen
  prev: Screen
  module: ModuleId | null
  report: Report | null
  path: string | null
  scanning: boolean
  progress: Progress | null
  error: string | null
}>({
  screen: 'home',
  prev: 'home',
  module: null,
  report: null,
  path: null,
  scanning: false,
  progress: null,
  error: null,
})

export function go(screen: Screen) {
  if (screen === nav.screen) return
  nav.prev = nav.screen
  nav.screen = screen
}

export function back() {
  if (nav.screen === 'module') return go('result')
  if (nav.screen === 'settings') return go(nav.prev === 'settings' ? 'home' : nav.prev)
  go('home')
}

export function openModule(id: ModuleId) {
  nav.module = id
  go('module')
}

// browser preview has no backend events, walk through the phases instead
function fakeProgress() {
  const steps: Progress[] = [
    { phase: 'plugins', done: 2, total: 6, current: 'LuckPerms' },
    { phase: 'plugins', done: 5, total: 6, current: 'Vulcan' },
    { phase: 'verify', done: 0, total: 6, current: '' },
    { phase: 'files', done: 20, total: 40, current: 'plugins/TAB/config.yml' },
  ]
  let i = 0
  const id = setInterval(() => (nav.progress = steps[Math.min(i++, steps.length - 1)]), 220)
  return () => clearInterval(id)
}

export async function startScan(path: string) {
  nav.scanning = true
  nav.error = null
  nav.progress = null
  const stop = inTauri ? await onScanProgress<Progress>((p) => (nav.progress = p)) : fakeProgress()
  try {
    const report = await scanServer(path)
    nav.progress = { phase: 'done', done: 1, total: 1, current: '' }
    // let the bar reach the end before the screen changes
    await new Promise((r) => setTimeout(r, 350))
    nav.report = report
    nav.path = path
    settings.lastPath = path
    go('result')
  } catch (e) {
    nav.error = friendlyError(e)
  } finally {
    stop()
    nav.scanning = false
  }
}

/** Re-reads the folder after a change so every screen shows what's on disk now. */
export async function rescan() {
  if (!nav.path) return
  try {
    nav.report = await scanServer(nav.path)
  } catch {
    // keep the old report on screen, the action itself already reported its result
  }
}
