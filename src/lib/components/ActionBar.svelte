<script lang="ts">
  import type { Snippet } from 'svelte'
  import Icon from './Icon.svelte'

  let { text, children }: { text: string; children: Snippet } = $props()
</script>

<div class="bar glass">
  <div class="state">
    <Icon name="shieldCheck" size={18} stroke={1.8} />
    <span>{text}</span>
  </div>
  <div class="actions">{@render children()}</div>
</div>

<style>
  .bar {
    position: fixed;
    left: 50%;
    bottom: 72px;
    transform: translateX(-50%);
    width: min(640px, calc(100vw - 48px));
    height: 60px;
    border-radius: 30px;
    padding: 0 10px 0 22px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    z-index: 15;
    border: 1px solid rgba(255, 255, 255, 0.14);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.1), 0 24px 70px rgba(0, 0, 0, 0.5);
  }
  .state {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    font-size: 13.5px;
    color: #c4c4cb;
  }
  .state span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .state :global(svg) {
    color: var(--green);
    flex: none;
  }
  .actions {
    display: flex;
    gap: 8px;
    flex: none;
  }
  .actions :global(.btn:disabled) {
    opacity: 0.45;
    cursor: default;
  }
</style>
