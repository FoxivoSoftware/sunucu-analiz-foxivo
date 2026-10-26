import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { inTauri } from './native'
import { settings } from './settings.svelte'
import { t } from './i18n.svelte'

export type Severity = 'high' | 'medium' | 'low' | 'info'

export interface Plugin {
  file: string
  name: string
  version: string | null
  apiVersion: string | null
  authors: string[]
  depend: string[]
  website: string | null
  official: { source: string; version: string; url: string; latest: string | null } | null
}
export interface Finding {
  plugin: string
  file: string
  rule: string
  severity: Severity
  detail: string
  official: boolean
}
export interface Hit {
  value: string
  count: number
  files: string[]
  detected: boolean
}
export interface EncodingIssue {
  file: string
  kind: 'mojibake' | 'legacy'
  lines: number
  before: string
  after: string
}
export interface Suggestion {
  id: string
  file: string
  key: string
  current: string
  recommended: string
  gameplay: boolean
  applicable: boolean
}
export interface Anticheat {
  name: string
  file: string
  version: string | null
}
export interface Report {
  root: string
  name: string
  platform: string
  mcVersion: string | null
  plugins: Plugin[]
  security: Finding[]
  placeholders: Hit[]
  encoding: EncodingIssue[]
  optimize: Suggestion[]
  anticheats: Anticheat[]
  running: boolean
  score: number
  breakdown: { id: string; points: number }[]
  filesScanned: number
  durationMs: number
}
export interface Progress {
  phase: 'plugins' | 'verify' | 'files' | 'done'
  done: number
  total: number
  current: string
}
export interface Preview {
  file: string
  changes: number
  before: string
  after: string
}
export interface Backup {
  id: string
  root: string
  label: 'replace' | 'encoding' | 'optimize'
  created: number
  files: string[]
  status: 'pending' | 'done' | 'rolledBack' | 'restored'
}
export type FixRequest =
  | { kind: 'replace'; values: Record<string, string> }
  | { kind: 'encoding'; files: string[] }
  | { kind: 'optimize'; ids: string[] }

/** Errors from Rust arrive as plain strings. */
export const errorText = (e: unknown) => (typeof e === 'string' ? e : e instanceof Error ? e.message : String(e))

export class DesktopOnly extends Error {}

// the core speaks Turkish; map the known cases to the UI language
export function friendlyError(e: unknown): string {
  if (e instanceof DesktopOnly) return t('errors.desktop')
  const msg = errorText(e)
  if (msg.includes('benzemiyor')) return t('errors.notServer')
  if (msg.includes('klasör bulunamadı')) return t('errors.notFound')
  if (msg.includes('çalışıyor')) return t('errors.running')
  if (msg.includes('doğrulanamadı')) return t('errors.invalid')
  if (msg.includes('korunan')) return t('errors.protected')
  if (msg.includes('kalmadı')) return t('plan.nothing')
  return t('errors.generic', { msg })
}

function desktop() {
  if (!inTauri) throw new DesktopOnly()
}

export async function scanServer(path: string): Promise<Report> {
  if (!inTauri && import.meta.env.DEV) {
    // lets the UI be worked on in a browser with a real report from the CLI
    const sample = (await import('./fixtures/sample-report.json')).default as Report
    await new Promise((r) => setTimeout(r, 900))
    return sample
  }
  desktop()
  return invoke<Report>('scan_server', { path, online: settings.verifyOnline })
}

export async function previewFix(path: string, fix: FixRequest) {
  desktop()
  return invoke<Preview[]>('preview_fix', { path, fix })
}

export async function applyFix(path: string, fix: FixRequest) {
  desktop()
  return invoke<Backup>('apply_fix', { path, backupDir: settings.backupDir, fix })
}

export async function listBackups(path: string) {
  if (!inTauri) return []
  return invoke<Backup[]>('list_backups', { path, backupDir: settings.backupDir })
}

export async function restoreBackup(path: string, id: string) {
  desktop()
  return invoke<Backup>('restore_backup', { path, backupDir: settings.backupDir, id })
}

export async function cleanupBackups() {
  if (!inTauri) return 0
  return invoke<number>('cleanup_backups', { backupDir: settings.backupDir, days: settings.retentionDays })
}

export async function openBackupDir() {
  desktop()
  return invoke<void>('open_backup_dir', { backupDir: settings.backupDir })
}

/** Asks where to save, then writes. Returns false if the user cancelled. */
export async function saveFile(defaultName: string, contents: string, ext: 'md' | 'json') {
  desktop()
  const path = await save({ defaultPath: defaultName, filters: [{ name: ext.toUpperCase(), extensions: [ext] }] })
  if (!path) return false
  await invoke('save_text', { path, contents })
  return true
}
