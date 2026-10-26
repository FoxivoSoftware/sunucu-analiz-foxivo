<script lang="ts">
  export type Mood = 'home' | 'warn' | 'critical' | 'clean' | 'calm'

  let { mood, animate = true }: { mood: Mood; animate?: boolean } = $props()
</script>

<div class="glow {mood}" class:still={!animate} aria-hidden="true">
  <div class="aurora">
    <i class="blob b1"></i>
    <i class="blob b2"></i>
    <i class="blob b3"></i>
    <i class="blob b4"></i>
  </div>
  <div class="horizon"></div>
  <div class="orb"></div>
  <div class="vignette"></div>
  <div class="grain"></div>
</div>

<style>
  /* registered so mood changes cross-fade instead of snapping */
  @property --c1 { syntax: '<color>'; inherits: true; initial-value: transparent; }
  @property --c2 { syntax: '<color>'; inherits: true; initial-value: transparent; }
  @property --c3 { syntax: '<color>'; inherits: true; initial-value: transparent; }
  @property --c4 { syntax: '<color>'; inherits: true; initial-value: transparent; }
  @property --horizon { syntax: '<color>'; inherits: true; initial-value: transparent; }
  @property --orb { syntax: '<color>'; inherits: true; initial-value: transparent; }

  .glow {
    position: fixed;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
    background: var(--bg);
    transition: --c1 1.8s ease, --c2 1.8s ease, --c3 1.8s ease, --c4 1.8s ease, --horizon 1.8s ease, --orb 1.8s ease;
  }

  /* kept quiet on purpose: the glow is a mood, not a feature */
  .home {
    --c1: rgba(99, 102, 241, 0.26);
    --c2: rgba(168, 120, 255, 0.18);
    --c3: rgba(236, 120, 170, 0.12);
    --c4: rgba(70, 120, 230, 0.16);
    --horizon: rgba(130, 110, 240, 0.07);
    --orb: rgba(129, 140, 248, 0.08);
  }
  .warn {
    --c1: rgba(214, 110, 50, 0.24);
    --c2: rgba(250, 170, 100, 0.15);
    --c3: rgba(236, 120, 150, 0.12);
    --c4: rgba(160, 70, 100, 0.16);
    --horizon: rgba(250, 150, 90, 0.07);
    --orb: rgba(250, 150, 90, 0.08);
  }
  .critical {
    --c1: rgba(210, 70, 80, 0.24);
    --c2: rgba(250, 120, 110, 0.15);
    --c3: rgba(240, 100, 150, 0.11);
    --c4: rgba(140, 40, 70, 0.17);
    --horizon: rgba(250, 100, 100, 0.07);
    --orb: rgba(250, 100, 100, 0.08);
  }
  .clean {
    --c1: rgba(40, 170, 110, 0.22);
    --c2: rgba(120, 220, 170, 0.14);
    --c3: rgba(80, 200, 200, 0.1);
    --c4: rgba(40, 120, 120, 0.15);
    --horizon: rgba(100, 210, 160, 0.06);
    --orb: rgba(100, 210, 160, 0.07);
  }
  .calm {
    --c1: rgba(99, 102, 241, 0.14);
    --c2: rgba(168, 120, 255, 0.09);
    --c3: rgba(236, 120, 170, 0.06);
    --c4: rgba(70, 120, 230, 0.09);
    --horizon: rgba(130, 110, 240, 0.04);
    --orb: rgba(129, 140, 248, 0.04);
  }

  .aurora {
    position: absolute;
    left: 50%;
    bottom: -310px;
    width: 1760px;
    height: 720px;
    transform: translateX(-50%);
  }

  /* each blob is one of the original aurora gradients, blurred on its own
     so the drift only moves an already rasterized layer */
  .blob {
    position: absolute;
    border-radius: 50%;
    filter: blur(96px);
    will-change: transform, opacity;
  }
  .b1 {
    left: calc(12% - 211px);
    top: calc(66% - 245px);
    width: 422px;
    height: 490px;
    background: radial-gradient(closest-side, var(--c4), transparent 74%);
    animation: path1 19s ease-in-out infinite, pulse 7s ease-in-out infinite alternate;
  }
  .b2 {
    left: calc(30% - 528px);
    top: calc(55% - 324px);
    width: 1056px;
    height: 648px;
    background: radial-gradient(closest-side, var(--c1), transparent 72%);
    animation: path2 24s ease-in-out infinite, pulse 9s ease-in-out -3s infinite alternate;
  }
  .b3 {
    left: calc(56% - 774px);
    top: calc(44% - 317px);
    width: 1548px;
    height: 634px;
    background: radial-gradient(closest-side, var(--c2), transparent 72%);
    animation: path3 21s ease-in-out -8s infinite, pulse 11s ease-in-out -5s infinite alternate;
  }
  .b4 {
    left: calc(82% - 317px);
    top: calc(58% - 302px);
    width: 634px;
    height: 604px;
    background: radial-gradient(closest-side, var(--c3), transparent 72%);
    animation: path4 17s ease-in-out -4s infinite, pulse 8s ease-in-out -2s infinite alternate;
  }

  /* closed loops so the motion never jumps back */
  @keyframes path1 {
    0%, 100% { transform: translate3d(0, 0, 0) scale(1); }
    33% { transform: translate3d(160px, -60px, 0) scale(1.12); }
    66% { transform: translate3d(60px, 50px, 0) scale(0.94); }
  }
  @keyframes path2 {
    0%, 100% { transform: translate3d(0, 0, 0) scale(1); }
    25% { transform: translate3d(140px, -40px, 0) scale(1.06); }
    50% { transform: translate3d(220px, 30px, 0) scale(1.1); }
    75% { transform: translate3d(70px, 55px, 0) scale(0.96); }
  }
  @keyframes path3 {
    0%, 100% { transform: translate3d(0, 0, 0) scale(1); }
    30% { transform: translate3d(-180px, 40px, 0) scale(1.05); }
    65% { transform: translate3d(-60px, -50px, 0) scale(0.95); }
  }
  @keyframes path4 {
    0%, 100% { transform: translate3d(0, 0, 0) scale(1); }
    40% { transform: translate3d(-150px, -70px, 0) scale(1.15); }
    70% { transform: translate3d(-40px, 30px, 0) scale(1.02); }
  }
  @keyframes pulse {
    from { opacity: 1; }
    to { opacity: 0.75; }
  }

  .horizon {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 260px;
    background: linear-gradient(to top, var(--horizon), transparent);
    filter: blur(10px);
  }
  .orb {
    position: absolute;
    left: 50%;
    top: 420px;
    width: 820px;
    height: 360px;
    transform: translateX(-50%);
    background: radial-gradient(closest-side, var(--orb), transparent 72%);
    filter: blur(64px);
    animation: breathe 16s ease-in-out infinite alternate;
  }
  @keyframes breathe {
    from { opacity: 1; }
    to { opacity: 0.7; }
  }
  .vignette {
    position: absolute;
    inset: 0;
    background: radial-gradient(1100px 520px at 50% -10%, rgba(255, 255, 255, 0.07), transparent 70%);
  }
  .grain {
    position: absolute;
    inset: 0;
    opacity: 0.06;
    mix-blend-mode: overlay;
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='200' height='200'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='.9' numOctaves='2' stitchTiles='stitch'/></filter><rect width='200' height='200' filter='url(%23n)'/></svg>");
  }

  .still .blob,
  .still .orb {
    animation: none;
  }
  @media (prefers-reduced-motion: reduce) {
    .blob,
    .orb {
      animation: none;
    }
  }
</style>
