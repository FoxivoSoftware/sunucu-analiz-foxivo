<script lang="ts">
  import { Tween } from 'svelte/motion'
  import { cubicOut } from 'svelte/easing'

  let { done, total, size = 58 }: { done: number; total: number; size?: number } = $props()

  const value = new Tween(0, { duration: 1200, easing: cubicOut, delay: 200 })
  $effect(() => {
    value.target = total ? done / total : 0
  })

  const stroke = 6
  const r = $derived((size - stroke) / 2)
  const length = $derived(2 * Math.PI * r)
</script>

<span class="ring" style:width="{size}px" style:height="{size}px">
  <svg viewBox="0 0 {size} {size}" aria-hidden="true">
    <circle cx={size / 2} cy={size / 2} {r} stroke-width={stroke} class="track" />
    <circle
      cx={size / 2}
      cy={size / 2}
      {r}
      stroke-width={stroke}
      class="arc"
      stroke-dasharray={length}
      stroke-dashoffset={length * (1 - value.current)}
    />
  </svg>
  <b>{done}<small>/{total}</small></b>
</span>

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
    stroke: rgba(111, 211, 154, 0.15);
  }
  .arc {
    stroke: var(--green);
    stroke-linecap: round;
    filter: drop-shadow(0 0 5px rgba(127, 224, 176, 0.6));
  }
  b {
    position: relative;
    font-size: 16px;
    font-weight: 700;
    color: #e3f8ee;
    font-variant-numeric: tabular-nums;
  }
  small {
    font-size: 11px;
    font-weight: 600;
    color: #8fbfa8;
  }
</style>
