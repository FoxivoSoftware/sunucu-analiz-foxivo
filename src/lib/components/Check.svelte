<script lang="ts">
  import Icon from './Icon.svelte'

  let {
    checked = $bindable(),
    label,
    onchange,
  }: { checked: boolean; label: string; onchange?: (value: boolean) => void } = $props()

  function toggle() {
    checked = !checked
    onchange?.(checked)
  }
</script>

<button class="check" class:on={checked} role="checkbox" aria-checked={checked} aria-label={label} onclick={toggle}>
  {#if checked}<Icon name="check" size={14} stroke={2.6} />{/if}
</button>

<style>
  .check {
    width: 22px;
    height: 22px;
    border-radius: 7px;
    display: grid;
    place-items: center;
    flex: none;
    color: #0a0a0d;
    border: 1.5px solid rgba(255, 255, 255, 0.28);
    transition: background 0.15s, border-color 0.15s;
  }
  .check:hover {
    border-color: rgba(255, 255, 255, 0.5);
  }
  .on {
    background: var(--text);
    border-color: var(--text);
  }
</style>
