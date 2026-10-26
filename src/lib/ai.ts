import { t } from './i18n.svelte'
import { notify } from './toast.svelte'

// Prompts are only copied to the clipboard; the app never sends anything itself.
export async function copyPrompt(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    notify(t('security.ai.copied'), 'success')
  } catch {
    notify(t('errors.clipboard'), 'error')
  }
}
