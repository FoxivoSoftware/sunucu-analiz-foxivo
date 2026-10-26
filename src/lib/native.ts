// Thin wrappers so the UI also runs in a plain browser (`npm run dev`) without Tauri.
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { openUrl } from '@tauri-apps/plugin-opener'
import { open } from '@tauri-apps/plugin-dialog'

export const inTauri = '__TAURI_INTERNALS__' in window
export const isMac = /Mac/i.test(navigator.userAgent)

const win = () => getCurrentWindow()

export const minimize = () => inTauri && win().minimize()
export const toggleMaximize = () => inTauri && win().toggleMaximize()
export const close = () => (inTauri ? win().close() : window.close())

export async function isMaximized() {
  return inTauri ? win().isMaximized() : false
}

export async function onResize(cb: () => void) {
  if (!inTauri) return () => {}
  return win().onResized(cb)
}

export async function openLink(url: string) {
  if (inTauri) {
    await openUrl(url)
  } else {
    window.open(url, '_blank', 'noopener')
  }
}

export async function pickFolder(title: string): Promise<string | null> {
  if (!inTauri) return null
  const picked = await open({ directory: true, multiple: false, title })
  return typeof picked === 'string' ? picked : null
}

type DropEvent = { type: 'enter' | 'over' | 'leave' | 'drop'; paths: string[] }

export async function onFileDrop(cb: (e: DropEvent) => void) {
  if (!inTauri) return () => {}
  return getCurrentWebview().onDragDropEvent(({ payload }) => {
    cb({ type: payload.type, paths: 'paths' in payload ? payload.paths : [] })
  })
}

export async function installerLanguage(): Promise<'tr' | 'en' | null> {
  if (!inTauri) return null
  return invoke<'tr' | 'en' | null>('installer_language').catch(() => null)
}

export const folderName = (path: string) => path.replace(/[\\/]+$/, '').split(/[\\/]/).pop() ?? path

export async function onScanProgress<T>(cb: (p: T) => void) {
  if (!inTauri) return () => {}
  const { listen } = await import('@tauri-apps/api/event')
  return listen<T>('scan-progress', (e) => cb(e.payload))
}
