<script lang="ts" generics="T extends string">
  let {
    options,
    value = $bindable(),
    label,
  }: { options: { value: T; label: string }[]; value: T; label: string } = $props()

  function onKey(e: KeyboardEvent) {
    if (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight') return
    e.preventDefault()
    const i = options.findIndex((o) => o.value === value)
    const next = (i + (e.key === 'ArrowRight' ? 1 : options.length - 1)) % options.length
    value = options[next].value
    ;(e.currentTarget as HTMLElement).querySelectorAll('button')[next]?.focus()
  }
</script>

<div class="seg" role="radiogroup" aria-label={label} tabindex="-1" onkeydown={onKey}>
  {#each options as o (o.value)}
    <button
      role="radio"
      aria-checked={o.value === value}
      tabindex={o.value === value ? 0 : -1}
      class:on={o.value === value}
      onclick={() => (value = o.value)}>{o.label}</button
    >
  {/each}
</div>

<style>
  .seg {
    display: flex;
    padding: 3px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid var(--line);
    backdrop-filter: blur(20px);
    flex: none;
  }
  button {
    font-size: 12px;
    font-weight: 600;
    padding: 5px 13px;
    border-radius: 999px;
    color: var(--text-2);
    transition: background 0.18s var(--ease), color 0.18s;
  }
  button:hover:not(.on) {
    color: var(--text);
  }
  .on {
    background: rgba(255, 255, 255, 0.16);
    color: var(--text);
  }
</style>
