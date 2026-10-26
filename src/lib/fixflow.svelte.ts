import { applyFix, previewFix, friendlyError, type FixRequest, type Preview } from './api'
import { nav, rescan } from './nav.svelte'
import { notify } from './toast.svelte'
import { t } from './i18n.svelte'

/** preview -> user approves in PlanDialog -> apply with backup -> rescan */
export class FixFlow {
  previews = $state<Preview[]>([])
  open = $state(false)
  busy = $state(false)
  private request: FixRequest | null = null

  async start(fix: FixRequest) {
    if (!nav.path) return
    this.busy = true
    try {
      const previews = await previewFix(nav.path, fix)
      if (!previews.length) {
        notify(t('plan.nothing'))
        return
      }
      this.request = fix
      this.previews = previews
      this.open = true
    } catch (e) {
      notify(friendlyError(e), 'error')
    } finally {
      this.busy = false
    }
  }

  async confirm() {
    if (!nav.path || !this.request) return
    this.busy = true
    try {
      const backup = await applyFix(nav.path, this.request)
      this.open = false
      notify(t('plan.done', { n: backup.files.length }), 'success')
      await rescan()
    } catch (e) {
      notify(friendlyError(e), 'error')
    } finally {
      this.busy = false
    }
  }

  cancel() {
    this.open = false
  }
}
