<script lang="ts">
  import Icon from './Icon.svelte'
  import type { Tone } from '$lib/report'

  let { icon, tone = 'neutral', size = 58 }: { icon: string; tone?: Tone; size?: number } = $props()

  const tint = { red: '255, 138, 128', amber: '255, 184, 113', green: '127, 224, 176', neutral: '165, 180, 252' }
</script>

<!-- glow is a sibling behind the tile, so it never gets clipped by or painted over the tile -->
<span class="wrap" style:width="{size}px" style:height="{size}px" style:--c={tint[tone]}>
  <span class="glow" aria-hidden="true"></span>
  <span class="tile">
    <span class="draw"><Icon name={icon} size={Math.round(size * 0.46)} stroke={1.7} /></span>
  </span>
</span>

<style>
  .wrap {
    position: relative;
    flex: none;
    display: block;
    isolation: isolate;
  }
  .glow {
    position: absolute;
    inset: -12px;
    border-radius: 50%;
    background: radial-gradient(closest-side, rgba(var(--c), 0.26), transparent);
    filter: blur(8px);
    animation: breathe 3s ease-in-out infinite alternate;
  }
  .tile {
    position: relative;
    width: 100%;
    height: 100%;
    display: grid;
    place-items: center;
    border-radius: 30%;
    background:
      radial-gradient(120% 120% at 30% 15%, rgba(var(--c), 0.22), transparent 60%),
      #1d1d23;
    box-shadow:
      inset 0 0 0 1px rgba(var(--c), 0.3),
      inset 0 1px 0 rgba(255, 255, 255, 0.12),
      0 10px 30px rgba(0, 0, 0, 0.35);
    color: rgb(var(--c));
    animation: pop 0.55s var(--ease) both;
  }
  /* the icon traces itself in */
  .draw :global(svg) {
    display: block;
  }
  .draw :global(path),
  .draw :global(circle),
  .draw :global(rect) {
    stroke-dasharray: 90;
    stroke-dashoffset: 90;
    animation: draw 1.1s 0.15s var(--ease) forwards;
  }
  @keyframes draw {
    to {
      stroke-dashoffset: 0;
    }
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: scale(0.85);
    }
  }
  @keyframes breathe {
    from {
      opacity: 0.4;
    }
    to {
      opacity: 1;
    }
  }
</style>
