<script lang="ts">
  import { tick } from 'svelte'
  import { fade, fly } from 'svelte/transition'
  import { SvelteSet } from 'svelte/reactivity'
  import { t, raw } from '$lib/i18n.svelte'
  import { settings } from '$lib/settings.svelte'
  import { openLink } from '$lib/native'
  import { DISCORD_URL } from '$lib/links'
  import { findTopic, type Topic } from '$lib/help'
  import Icon from './Icon.svelte'
  import logo from '$lib/assets/logo.png'

  type Message = {
    id: number
    from: 'bot' | 'user'
    text: string
    typing?: boolean
    streaming?: boolean
    related?: string[]
  }

  let open = $state(false)
  let busy = $state(false)
  let draft = $state('')
  let messages = $state<Message[]>([])
  let list = $state<HTMLElement>()
  let input = $state<HTMLInputElement>()
  const asked = new SvelteSet<string>()
  let nextId = 0

  const CHARS_PER_SEC = 140

  const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms))
  const instant = () => settings.reduceMotion || matchMedia('(prefers-reduced-motion: reduce)').matches
  const topics = () => raw<Topic[]>('help.topics')

  // next few questions after the current one that haven't been asked yet
  function related(after: Topic | null, count = 3) {
    const all = topics().filter((tp) => tp.q)
    const start = after ? all.findIndex((tp) => tp.q === after.q) + 1 : 0
    const ordered = [...all.slice(start), ...all.slice(0, start)]
    return ordered.filter((tp) => !asked.has(tp.q) && tp.q !== after?.q).slice(0, count).map((tp) => tp.q)
  }

  function nearBottom() {
    return !list || list.scrollHeight - list.scrollTop - list.clientHeight < 80
  }

  async function scrollDown(force = false) {
    if (!force && !nearBottom()) return
    await tick()
    list?.scrollTo({ top: list.scrollHeight })
  }

  // dots for a moment, then the answer is written out at a steady pace
  async function say(text: string, follow: string[]) {
    messages.push({ id: nextId++, from: 'bot', text: '', typing: true })
    const live = messages[messages.length - 1]
    await scrollDown(true)

    if (!instant()) await sleep(450 + Math.random() * 350)
    live.typing = false

    if (instant()) {
      live.text = text
    } else {
      live.streaming = true
      const started = performance.now()
      await new Promise<void>((done) => {
        const step = (now: number) => {
          const n = Math.min(text.length, Math.round(((now - started) / 1000) * CHARS_PER_SEC))
          // don't cut a word in half
          const cut = n >= text.length ? n : text.lastIndexOf(' ', n) + 1 || n
          live.text = text.slice(0, cut)
          scrollDown()
          if (n < text.length) requestAnimationFrame(step)
          else done()
        }
        requestAnimationFrame(step)
      })
      live.streaming = false
    }
    live.related = follow
    await scrollDown()
  }

  async function ask(question: string) {
    const q = question.trim()
    if (!q || busy) return
    busy = true
    draft = ''
    for (const m of messages) m.related = undefined
    messages.push({ id: nextId++, from: 'user', text: q })

    const topic = findTopic(q, topics())
    if (topic?.q) asked.add(topic.q)
    await say(topic?.answer ?? t('help.fallback'), related(topic))

    busy = false
    input?.focus()
  }

  function greet() {
    messages = []
    asked.clear()
    say(t('help.greeting'), related(null, 4))
  }

  async function toggle() {
    open = !open
    if (!open) return
    if (messages.length === 0) greet()
    await tick()
    input?.focus()
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape' && open) open = false
  }
</script>

<svelte:window onkeydown={onKey} />

{#if open}
  <section
    class="panel"
    aria-label={t('help.title')}
    transition:fade={{ duration: instant() ? 0 : 200 }}
  >
    <header>
      <img class="avatar" src={logo} alt="" draggable="false" />
      <div class="who">
        <b>{t('help.title')}</b>
        <span>{t('help.subtitle')}</span>
      </div>
      <button class="hbtn" onclick={greet} disabled={busy} aria-label={t('help.newChat')} title={t('help.newChat')}>
        <Icon name="refresh" size={15} stroke={2} />
      </button>
      <button class="hbtn" onclick={() => (open = false)} aria-label={t('help.close')} title={t('help.closeShort')}>
        <Icon name="x" size={16} stroke={2} />
      </button>
    </header>

    <div class="messages" bind:this={list} aria-live="polite">
      {#each messages as m (m.id)}
        {#if m.from === 'user'}
          <div class="user" in:fly={{ y: 6, duration: instant() ? 0 : 200 }}>{m.text}</div>
        {:else}
          <div class="bot" in:fade={{ duration: instant() ? 0 : 180 }}>
            <img class="mini" src={logo} alt="" draggable="false" />
            <div class="body">
              {#if m.typing}
                <span class="thinking">{t('help.thinking')}<i></i><i></i><i></i></span>
              {:else}
                <p class:streaming={m.streaming}>{m.text}</p>
              {/if}
              {#if m.related?.length}
                <div class="related" in:fade={{ duration: instant() ? 0 : 220 }}>
                  {#each m.related as r (r)}
                    <button onclick={() => ask(r)}>{r}</button>
                  {/each}
                </div>
              {/if}
            </div>
          </div>
        {/if}
      {/each}
    </div>

    <form
      class="composer"
      onsubmit={(e) => {
        e.preventDefault()
        ask(draft)
      }}
    >
      <input bind:this={input} bind:value={draft} placeholder={t('help.placeholder')} maxlength="300" autocomplete="off" />
      <button class="send" type="submit" disabled={!draft.trim() || busy} aria-label={t('help.send')}>
        <Icon name="arrowUp" size={16} stroke={2.2} />
      </button>
    </form>

    <footer>
      <span>{t('help.offline')}</span>
      <button class="link" onclick={() => openLink(DISCORD_URL)}>
        {t('help.community')}
        <Icon name="external" size={11} stroke={2} />
      </button>
    </footer>
  </section>
{/if}

<button class="launcher" class:open onclick={toggle} aria-label={t(open ? 'help.close' : 'help.open')} aria-expanded={open}>
  <span class="ring" aria-hidden="true"></span>
  <span class="glyph" class:turned={open}>
    {#if open}
      <Icon name="x" size={17} stroke={2} />
    {:else}
      <Icon name="helpBubble" size={19} stroke={1.8} />
    {/if}
  </span>
  <span class="label">{t(open ? 'help.closeShort' : 'help.open')}</span>
</button>

<style>
  @property --spin {
    syntax: '<angle>';
    inherits: false;
    initial-value: 0deg;
  }

  /* launcher */
  .launcher {
    position: fixed;
    right: 22px;
    bottom: 15px;
    height: 48px;
    padding: 0 22px 0 16px;
    border-radius: 999px;
    display: flex;
    align-items: center;
    gap: 10px;
    z-index: 30;
    font-size: 14px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: #f5f5f7;
    background: rgba(30, 30, 38, 0.82);
    backdrop-filter: blur(24px) saturate(160%);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.08),
      0 12px 32px rgba(0, 0, 0, 0.4);
    animation: rise 0.7s var(--ease) 0.3s both;
    transition:
      transform 0.3s var(--ease),
      background 0.3s;
  }
  .launcher::after {
    /* soft halo that breathes, no hard edges */
    content: '';
    position: absolute;
    inset: -6px;
    border-radius: inherit;
    background: radial-gradient(closest-side, rgba(143, 124, 255, 0.12), transparent);
    filter: blur(12px);
    z-index: -1;
    animation: breathe 4s ease-in-out infinite alternate;
    pointer-events: none;
  }
  .ring {
    /* 1px gradient border that slowly turns */
    position: absolute;
    inset: 0;
    border-radius: inherit;
    padding: 1px;
    background: conic-gradient(from var(--spin), rgba(129, 140, 248, 0.32), rgba(192, 132, 252, 0.3), rgba(244, 143, 177, 0.26), rgba(255, 255, 255, 0.1) 60%, rgba(129, 140, 248, 0.32));
    mask:
      linear-gradient(#000 0 0) content-box,
      linear-gradient(#000 0 0);
    mask-composite: exclude;
    animation: spin 8s linear infinite;
    pointer-events: none;
  }
  .launcher:hover {
    background: rgba(40, 40, 50, 0.88);
  }
  .glyph {
    display: grid;
    transition: transform 0.4s var(--ease);
  }
  .glyph.turned {
    transform: rotate(90deg);
  }
  .launcher:not(.open):hover .glyph {
    transform: scale(1.08);
  }
  @keyframes spin {
    to {
      --spin: 360deg;
    }
  }
  @keyframes breathe {
    from {
      opacity: 0.55;
    }
    to {
      opacity: 1;
    }
  }
  @keyframes rise {
    from {
      opacity: 0;
    }
  }

  /* panel */
  .panel {
    position: fixed;
    right: 22px;
    bottom: 76px;
    width: 384px;
    height: min(560px, calc(100vh - 160px));
    transform-origin: bottom right;
    border-radius: 26px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    z-index: 30;
    background: rgba(24, 24, 30, 0.82);
    backdrop-filter: blur(40px) saturate(170%);
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.08),
      0 30px 80px rgba(0, 0, 0, 0.55),
      0 0 60px rgba(143, 124, 255, 0.08);
  }

  header {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 14px 10px 12px 14px;
  }
  .avatar {
    width: 38px;
    height: 38px;
    border-radius: 11px;
    display: block;
    box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.1);
    flex: none;
  }
  .who {
    flex: 1;
    min-width: 0;
  }
  .who b {
    display: block;
    font-size: 14.5px;
    font-weight: 600;
    letter-spacing: -0.015em;
  }
  .who span {
    display: block;
    margin-top: 1px;
    font-size: 12px;
    color: var(--text-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .hbtn {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: var(--text-2);
    flex: none;
    transition: background 0.15s, color 0.15s;
  }
  .hbtn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text);
  }
  .hbtn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 6px 16px 10px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.12) transparent;
    mask-image: linear-gradient(to bottom, transparent 0, #000 12px, #000 calc(100% - 12px), transparent 100%);
  }

  .user {
    align-self: flex-end;
    max-width: 82%;
    padding: 9px 14px;
    border-radius: 18px 18px 6px 18px;
    font-size: 13.5px;
    line-height: 1.45;
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
    overflow-wrap: anywhere;
  }

  .bot {
    display: flex;
    gap: 10px;
    align-items: flex-start;
  }
  .mini {
    width: 24px;
    height: 24px;
    border-radius: 7px;
    display: block;
    box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.08);
    flex: none;
    margin-top: 1px;
  }
  .body {
    flex: 1;
    min-width: 0;
  }
  .body p {
    font-size: 13.5px;
    line-height: 1.6;
    letter-spacing: -0.005em;
    color: #e2e2e7;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .streaming::after {
    content: '';
    display: inline-block;
    width: 7px;
    height: 7px;
    margin-left: 4px;
    border-radius: 50%;
    background: linear-gradient(135deg, #a5b4fc, #f0abfc);
    vertical-align: middle;
    animation: pulse 0.9s ease-in-out infinite alternate;
  }
  @keyframes pulse {
    from {
      opacity: 0.35;
      transform: scale(0.8);
    }
  }

  .thinking {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 13px;
    font-weight: 500;
    background: linear-gradient(90deg, #6f6f78 0%, #d7d7de 40%, #6f6f78 80%);
    background-size: 200% 100%;
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
    animation: shimmer 1.6s linear infinite;
    padding-top: 3px;
  }
  .thinking i {
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: #9a9aa2;
    animation: pulse 0.6s ease-in-out infinite alternate;
  }
  .thinking i:nth-of-type(2) {
    animation-delay: 0.2s;
  }
  .thinking i:nth-of-type(3) {
    animation-delay: 0.4s;
  }
  @keyframes shimmer {
    from {
      background-position: 100% 0;
    }
    to {
      background-position: -100% 0;
    }
  }

  .related {
    margin-top: 10px;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
  }
  .related button {
    min-height: 30px;
    padding: 6px 12px;
    border-radius: 12px;
    font-size: 12.5px;
    font-weight: 500;
    text-align: left;
    color: #cfcfd6;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.03);
    transition: background 0.15s, border-color 0.15s, color 0.15s;
  }
  .related button:hover {
    color: #fff;
    background: rgba(143, 124, 255, 0.12);
    border-color: rgba(143, 124, 255, 0.35);
  }

  .composer {
    margin: 4px 12px 0;
    height: 46px;
    padding: 0 6px 0 16px;
    display: flex;
    align-items: center;
    gap: 8px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    transition: border-color 0.2s, box-shadow 0.2s;
  }
  .composer:focus-within {
    border-color: rgba(143, 124, 255, 0.45);
    box-shadow: 0 0 0 4px rgba(143, 124, 255, 0.1);
  }
  .composer input {
    flex: 1;
    min-width: 0;
    height: 100%;
    background: none;
    border: 0;
    outline: none;
    color: var(--text);
    font: inherit;
    font-size: 13.5px;
  }
  .composer input::placeholder {
    color: var(--text-3);
  }
  .send {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--text);
    color: #0a0a0d;
    flex: none;
    transition: opacity 0.15s, transform 0.15s var(--ease);
  }
  .send:not(:disabled):hover {
    transform: scale(1.06);
  }
  .send:disabled {
    opacity: 0.3;
    cursor: default;
  }

  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 10px 18px 13px;
    font-size: 11px;
    color: var(--text-3);
  }
  .link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-2);
    flex: none;
  }
  .link:hover {
    color: var(--text);
  }
</style>
