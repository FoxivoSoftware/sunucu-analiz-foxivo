export type Lang = 'tr' | 'en'
export type UiScale = 'compact' | 'auto' | 'large'

export interface Settings {
  lang: Lang
  uiScale: UiScale
  glow: boolean
  reduceMotion: boolean
  backupDir: string
  retentionDays: number
  /** look plugin jars up on Modrinth by hash */
  verifyOnline: boolean
  /** closing the window keeps the app in the tray */
  closeToTray: boolean
  autoUpdate: boolean
  lastPath: string | null
  /** values typed into "replace sample values", reused for the next server */
  replaceValues: Record<string, string>
}

const KEY = 'settings'

const defaults: Settings = {
  lang: navigator.language.toLowerCase().startsWith('tr') ? 'tr' : 'en',
  uiScale: 'auto',
  glow: true,
  reduceMotion: false,
  backupDir: '~/Foxivo/Yedekler',
  retentionDays: 30,
  verifyOnline: true,
  closeToTray: true,
  autoUpdate: true,
  lastPath: null,
  replaceValues: {},
}

function stored(): Partial<Settings> | null {
  try {
    const raw = localStorage.getItem(KEY)
    return raw ? JSON.parse(raw) : null
  } catch {
    return null
  }
}

const saved = stored()

/** Nothing saved yet, so the installer's language choice still applies. */
export const firstRun = saved === null

export const settings = $state<Settings>({ ...defaults, ...saved })

export const RETENTION_OPTIONS = [7, 14, 30, 60, 90]

$effect.root(() => {
  $effect(() => {
    try {
      localStorage.setItem(KEY, JSON.stringify(settings))
    } catch {
      // storage can be unavailable, settings just won't persist
    }
  })
})
