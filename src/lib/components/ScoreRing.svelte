<script lang="ts">
  import { Tween } from 'svelte/motion'
  import { cubicOut } from 'svelte/easing'

  let { score, size = 148 }: { score: number; size?: number } = $props()

  const value = new Tween(0, { duration: 1400, easing: cubicOut, delay: 150 })
  $effect(() => {
    value.target = score
  })

  const id = $props.id()
  const stroke = 11
  const r = $derived((size - stroke) / 2 - 4)
  const length = $derived(2 * Math.PI * r)

  // red under 40, warm under 75, green above
  const colors = $derived(
    score < 40 ? ['#ff8a80', '#f472b6'] : score < 75 ? ['#fdba74', '#f9a8d4'] : ['#86efac', '#5eead4'],
  )
</script>

<div class="ring" style:width="{size}px" style:height="{size}px" style:--a={colors[0]} style:--b={colors[1]}>
  <span class="halo" aria-hidden="true"></span>
  <span class="sheen" aria-hidden="true"></span>
  <svg viewBox="0 0 {size} {size}" aria-hidden="true">
    <defs>
      <linearGradient id="g-{id}" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color={colors[0]} />
        <stop offset="1" stop-color={colors[1]} />
      </linearGradient>
    </defs>
    <circle class="track" cx={size / 2} cy={size / 2} {r} stroke-width={stroke} />
    <circle
      class="arc"
      cx={size / 2}
      cy={size / 2}
      {r}
      stroke-width={stroke}
      stroke="url(#g-{id})"
      stroke-dasharray={length}
      stroke-dashoffset={length * (1 - value.current / 100)}
    />
  </svg>
  <div class="label">
    <b>{Math.round(value.current)}</b>
    <span>/100</span>
  </div>
</div>

<style>
  .ring {
    position: relative;
    flex: none;
    display: grid;
    place-items: center;
  }
  svg {
    position: absolute;
    inset: 0;
    overflow: visible; /* the arc glow would be cut at the box edge */
    transform: rotate(-90deg);
  }
  circle {
    fill: none;
  }
  .track {
    stroke: rgba(255, 255, 255, 0.08);
  }
  .arc {
    stroke-linecap: round;
    filter: drop-shadow(0 0 6px var(--a));
  }
  .halo {
    position: absolute;
    inset: 14%;
    border-radius: 50%;
    background: radial-gradient(closest-side, var(--a), transparent);
    opacity: 0.22;
    filter: blur(18px);
    animation: breathe 3.2s ease-in-out infinite alternate;
  }
  .sheen {
    position: absolute;
    inset: 9px;
    border-radius: 50%;
    background: conic-gradient(from 0deg, transparent 0 70%, rgba(255, 255, 255, 0.09) 85%, transparent 100%);
    mask: radial-gradient(closest-side, transparent calc(100% - 13px), #000 calc(100% - 12px));
    animation: spin 6s linear infinite;
  }
  .label {
    position: relative;
    display: flex;
    align-items: baseline;
    gap: 2px;
  }
  .label b {
    font-size: 44px;
    font-weight: 700;
    letter-spacing: -0.04em;
    font-variant-numeric: tabular-nums;
    color: #fff;
  }
  .label span {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-3);
  }
  @keyframes breathe {
    from {
      opacity: 0.14;
      transform: scale(0.92);
    }
    to {
      opacity: 0.3;
      transform: scale(1.06);
    }
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
