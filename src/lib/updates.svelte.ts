// App updates from GitHub releases. The updater only installs packages whose
// signature matches the public key built into the app.
import type { Update } from '@tauri-apps/plugin-updater'
import { inTauri } from './native'

type Status = 'idle' | 'checking' | 'latest' | 'available' | 'downloading' | 'error'

export const updates = $state<{ status: Status; version: string; progress: number; dismissed: boolean }>({
  status: 'idle',
  version: '',
  progress: 0,
  dismissed: false,
})

let pending: Update | null = null

/** `quiet` is for the startup check: no error state if GitHub can't be reached. */
export async function checkUpdates(quiet = false) {
  if (!inTauri || updates.status === 'checking' || updates.status === 'downloading') return
  updates.status = 'checking'
  try {
    const { check } = await import('@tauri-apps/plugin-updater')
    pending = await check()
    if (pending) {
      updates.version = pending.version
      updates.status = 'available'
    } else {
      updates.status = quiet ? 'idle' : 'latest'
    }
  } catch {
    updates.status = quiet ? 'idle' : 'error'
  }
}

export async function installUpdate() {
  if (!pending) return
  updates.status = 'downloading'
  updates.progress = 0
  let total = 0
  let done = 0
  try {
    await pending.downloadAndInstall((event) => {
      if (event.event === 'Started') total = event.data.contentLength ?? 0
      else if (event.event === 'Progress') {
        done += event.data.chunkLength
        updates.progress = total ? done / total : 0
      }
    })
    const { relaunch } = await import('@tauri-apps/plugin-process')
    await relaunch()
  } catch {
    updates.status = 'error'
  }
}
