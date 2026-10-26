export type ToastTone = 'info' | 'success' | 'error'

export const toast = $state<{ text: string; tone: ToastTone; key: number }>({ text: '', tone: 'info', key: 0 })

let timer: ReturnType<typeof setTimeout> | undefined

export function notify(text: string, tone: ToastTone = 'info') {
  toast.text = text
  toast.tone = tone
  toast.key++
  clearTimeout(timer)
  timer = setTimeout(() => (toast.text = ''), tone === 'error' ? 6000 : 3800)
}
