import { t } from './i18n.svelte'
import type { Report } from './api'

export type ModuleId = 'security' | 'replace' | 'encoding' | 'optimize' | 'anticheat' | 'backups'
export type Tone = 'red' | 'amber' | 'green' | 'neutral'

export interface Card {
  id: ModuleId
  icon: string
  tone: Tone
  badge: string
  hint: string
}

// findings inside byte-identical official releases are the author's own code, not a risk signal
const open = (r: Report) => r.security.filter((f) => !f.official && f.severity !== 'info')
const count = (r: Report, sev: string) => open(r).filter((f) => f.severity === sev).length

export function cards(r: Report): Card[] {
  const high = count(r, 'high')
  const findings = open(r)
  const verified = r.plugins.filter((p) => p.official).length
  const spots = r.placeholders.reduce((n, h) => n + h.count, 0)
  const ac = r.anticheats

  return [
    {
      id: 'security',
      icon: 'shieldCheck',
      tone: high ? 'red' : findings.length ? 'amber' : 'green',
      badge: high
        ? t('cards.high', { n: high })
        : findings.length
          ? t('cards.findings', { n: findings.length })
          : t('cards.clean'),
      hint: findings.length
        ? t('cards.securityHint', { file: findings[0].file })
        : verified
          ? t('cards.securityVerified', { n: verified, total: r.plugins.length })
          : t('cards.securityClean', { n: r.plugins.length }),
    },
    {
      id: 'replace',
      icon: 'swap',
      tone: spots ? 'amber' : 'green',
      badge: spots ? t('cards.spots', { n: spots }) : t('cards.clean'),
      hint: spots
        ? r.placeholders.length > 1
          ? t('cards.replaceHint', { value: r.placeholders[0].value, n: r.placeholders.length - 1 })
          : r.placeholders[0].value
        : t('cards.replaceClean'),
    },
    {
      id: 'encoding',
      icon: 'type',
      tone: r.encoding.length ? 'amber' : 'green',
      badge: r.encoding.length ? t('cards.files', { n: r.encoding.length }) : t('cards.clean'),
      hint: r.encoding.length ? t('cards.encodingHint') : t('cards.encodingClean'),
    },
    {
      id: 'optimize',
      icon: 'gauge',
      tone: r.optimize.length ? 'neutral' : 'green',
      badge: r.optimize.length ? t('cards.tips', { n: r.optimize.length }) : t('cards.good'),
      hint: r.optimize.length ? t('cards.optimizeHint') : t('cards.optimizeClean'),
    },
    {
      id: 'anticheat',
      icon: 'shieldAlert',
      tone: ac.length > 1 ? 'amber' : ac.length ? 'green' : 'neutral',
      badge: ac.length ? t('cards.systems', { n: ac.length }) : t('cards.none'),
      hint:
        ac.length > 1
          ? t('cards.anticheatMany', { a: ac[0].name, b: ac[1].name })
          : ac.length
            ? ac[0].name
            : t('cards.anticheatNone'),
    },
    {
      id: 'backups',
      icon: 'history',
      tone: 'green',
      badge: t('cards.ready'),
      hint: t('cards.backupsHint'),
    },
  ]
}

/** How many cards need the user's attention. */
export const issueCount = (r: Report) =>
  cards(r).filter((c) => c.id !== 'backups' && (c.tone === 'red' || c.tone === 'amber' || (c.id === 'optimize' && c.tone === 'neutral')))
    .length

/** The one thing we'd do first, in order of risk. */
export function nextStep(r: Report): ModuleId | null {
  if (open(r).some((f) => f.severity === 'high' || f.severity === 'medium')) return 'security'
  if (r.placeholders.length) return 'replace'
  if (r.encoding.length) return 'encoding'
  if (r.optimize.length) return 'optimize'
  if (r.anticheats.length > 1) return 'anticheat'
  return null
}
