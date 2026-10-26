<script lang="ts">
  import { t } from '$lib/i18n.svelte'
  import { openLink } from '$lib/native'
  import { FOXIVO_URL } from '$lib/links'
  import Logo from './Logo.svelte'
</script>

<footer>
  <button class="pill" onclick={() => openLink(FOXIVO_URL)}>
    <Logo width={22} />
    <span>{t('foxivo.poweredBy')} <b>Foxivo</b></span>
  </button>
</footer>

<style>
  footer {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 22px;
    display: flex;
    justify-content: center;
    z-index: 10;
    pointer-events: none;
  }
  .pill {
    pointer-events: auto;
    height: 34px;
    padding: 0 16px 0 13px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    gap: 10px;
    font-size: 13.5px;
    font-weight: 500;
    letter-spacing: -0.005em;
    color: #d1d1d6;
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.12);
    backdrop-filter: blur(20px);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
    transition: background 0.2s, border-color 0.2s, color 0.2s;
  }
  .pill:hover {
    background: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.18);
    color: #fff;
  }
  .pill b {
    font-weight: 700;
    color: #fff;
    letter-spacing: -0.01em;
  }
</style>
