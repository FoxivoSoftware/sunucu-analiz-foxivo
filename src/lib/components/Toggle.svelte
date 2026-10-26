<script lang="ts">
  let { checked = $bindable(), label }: { checked: boolean; label: string } = $props()
</script>

<button class="tg" class:on={checked} role="switch" aria-checked={checked} aria-label={label} onclick={() => (checked = !checked)}>
  <i></i>
</button>

<style>
  .tg {
    width: 44px;
    height: 26px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.14);
    position: relative;
    flex: none;
    transition: background 0.2s;
  }
  .tg i {
    position: absolute;
    left: 3px;
    top: 3px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
    transition: transform 0.22s var(--ease);
  }
  .on {
    background: #34c98a;
  }
  .on i {
    transform: translateX(18px);
  }
</style>
