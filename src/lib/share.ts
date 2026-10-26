import { saveFile, friendlyError, type Report } from './api'
import { t } from './i18n.svelte'
import { settings } from './settings.svelte'
import { notify } from './toast.svelte'
import { cards } from './report'

// folder paths can contain the Windows user name, never put them in a shared file
const mask = (text: string, r: Report) => text.split(r.root).join('<sunucu>').replace(/[A-Z]:\\Users\\[^\\/]+/gi, '<kullanıcı>')

function markdown(r: Report) {
  const lines = [
    `# ${t('share.title')}: ${r.name}`,
    '',
    `- ${r.platform}${r.mcVersion ? ` ${r.mcVersion}` : ''} · ${t('result.plugins', { n: r.plugins.length })}`,
    `- ${t('share.score')}: **${r.score}/100**`,
    `- ${t('share.date')}: ${new Date().toLocaleString(settings.lang)}`,
    '',
    `## ${t('share.summary')}`,
    '',
    ...cards(r).filter((c) => c.id !== 'backups').map((c) => `- **${t(`modules.${c.id}`)}:** ${c.badge} · ${c.hint}`),
    '',
  ]
  if (r.security.length) {
    lines.push(`## ${t('modules.security')}`, '')
    for (const f of r.security) {
      lines.push(`- [${t(`sev.${f.severity}`)}] ${f.file}: ${t(`security.rules.${f.rule}.title`)} (${f.detail})`)
    }
    lines.push('', `> ${t('security.disclaimer')}`, '')
  }
  if (r.placeholders.length) {
    lines.push(`## ${t('modules.replace')}`, '')
    for (const h of r.placeholders) lines.push(`- \`${h.value}\`: ${t('replace.spots', { n: h.count, files: h.files.length })}`)
    lines.push('')
  }
  if (r.encoding.length) {
    lines.push(`## ${t('modules.encoding')}`, '')
    for (const e of r.encoding) lines.push(`- ${e.file} (${t('encoding.lines', { n: e.lines })})`)
    lines.push('')
  }
  if (r.optimize.length) {
    lines.push(`## ${t('modules.optimize')}`, '')
    for (const s of r.optimize) lines.push(`- ${t(`optimize.tips.${s.id}.title`)}: ${s.applicable ? `${s.key} ${s.current} → ${s.recommended}` : s.file}`)
    lines.push('')
  }
  lines.push(`## ${t('share.plugins')}`, '')
  for (const p of r.plugins) lines.push(`- ${p.name} ${p.version ?? ''}`.trimEnd())
  lines.push('', `---`, `${t('app.name')} · Powered by Foxivo · foxivo.com.tr`)
  return mask(lines.join('\n'), r)
}

export async function shareReport(r: Report) {
  try {
    const saved = await saveFile(`${r.name}-rapor.md`, markdown(r), 'md')
    if (saved) notify(t('share.saved'), 'success')
  } catch (e) {
    notify(friendlyError(e), 'error')
  }
}

export async function exportDiagnostics(r: Report | null) {
  const data = {
    app: t('app.name'),
    version: __APP_VERSION__,
    platform: navigator.userAgent,
    language: settings.lang,
    uiScale: settings.uiScale,
    lastScan: r && {
      platform: r.platform,
      mcVersion: r.mcVersion,
      plugins: r.plugins.map((p) => `${p.name} ${p.version ?? ''}`.trim()),
      score: r.score,
      findings: r.security.map((f) => `${f.rule}:${f.file}`),
      filesScanned: r.filesScanned,
      durationMs: r.durationMs,
    },
  }
  try {
    const text = JSON.stringify(data, null, 2)
    const saved = await saveFile('tanilama.json', r ? mask(text, r) : text, 'json')
    if (saved) notify(t('share.saved'), 'success')
  } catch (e) {
    notify(friendlyError(e), 'error')
  }
}
