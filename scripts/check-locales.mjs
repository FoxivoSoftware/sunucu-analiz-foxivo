// Fails if tr.json and en.json don't have the same keys.
import { readFileSync } from 'node:fs'

const load = (lang) => JSON.parse(readFileSync(new URL(`../locales/${lang}.json`, import.meta.url), 'utf8'))

// string arrays (keywords, suggestions) can differ per language, so they count as one value
const isLeaf = (v) => !v || typeof v !== 'object' || (Array.isArray(v) && v.every((x) => typeof x === 'string'))

function keys(obj, prefix = '') {
  return Object.entries(obj).flatMap(([k, v]) => (isLeaf(v) ? [`${prefix}${k}`] : keys(v, `${prefix}${k}.`)))
}

const tr = new Set(keys(load('tr')))
const en = new Set(keys(load('en')))
const missingEn = [...tr].filter((k) => !en.has(k))
const missingTr = [...en].filter((k) => !tr.has(k))

if (missingEn.length || missingTr.length) {
  if (missingEn.length) console.error('en.json is missing:\n  ' + missingEn.join('\n  '))
  if (missingTr.length) console.error('tr.json is missing:\n  ' + missingTr.join('\n  '))
  process.exit(1)
}
console.log(`locales ok (${tr.size} keys)`)
