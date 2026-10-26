import tr from '$locales/tr.json'
import en from '$locales/en.json'
import { settings, type Lang } from './settings.svelte'

const dicts: Record<Lang, unknown> = { tr, en }

function resolve(root: unknown, key: string): unknown {
  let node = root
  for (const part of key.split('.')) {
    if (!node || typeof node !== 'object') return undefined
    node = (node as Record<string, unknown>)[part]
  }
  return node
}

/** Non-string entries (lists, help topics) for the current language. */
export function raw<T>(key: string): T {
  return (resolve(dicts[settings.lang], key) ?? resolve(dicts.tr, key)) as T
}

export function t(key: string, params?: Record<string, string | number>): string {
  const found = resolve(dicts[settings.lang], key) ?? resolve(dicts.tr, key)
  const text = typeof found === 'string' ? found : key
  if (!params) return text
  return text.replace(/\{(\w+)\}/g, (m, name) => (name in params ? String(params[name]) : m))
}

$effect.root(() => {
  $effect(() => {
    document.documentElement.lang = settings.lang
    document.title = t('app.name')
  })
})
