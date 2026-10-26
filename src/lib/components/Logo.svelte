<script lang="ts">
  // Vector version of assets/logo.png, traced from the 512px original.
  let { width = 32 }: { width?: number } = $props()
</script>

<svg {width} height={(width * 102) / 250} viewBox="0 0 250 102" stroke-width="10" stroke-linejoin="round" aria-hidden="true">
  <path d="M21.2 5H32.2L16.8 97H5.8Z" fill="#00F2FE" stroke="#00F2FE" />
  <path d="M63.2 5H110.2L94.8 97H47.8Z" fill="#4FACFE" stroke="#4FACFE" />
  <path d="M140.2 5H244.2L228.8 97H124.8Z" fill="#0052CC" stroke="#0052CC" />
</svg>

<style>
  svg {
    display: block;
    flex: none;
    overflow: visible;
  }
</style>
