<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import StatusPill from "./StatusPill.svelte";

  export let state: string = "idle";
  export let subState: string | null = null;
  export let cycle: number = 0;
  export let sessionSp: number = 0;
  export let sessionCr: number = 0;
  export let sessionWspins: number = 0;
  export let connected: boolean = false;

  const dispatch = createEventDispatcher<{
    settings: null;
  }>();

  function formatCr(n: number): string {
    const sign = n < 0 ? "-" : "";
    const absN = Math.abs(n);
    if (absN >= 1_000_000)
      return sign + (absN / 1_000_000).toFixed(1).replace(".0", "") + "M";
    if (absN >= 1_000) return sign + (absN / 1_000).toFixed(0) + "K";
    return sign + String(absN);
  }
</script>

<header class="bot-header">
  <!-- Left: Logo + Status -->
  <div class="header-left">
    <div class="logo-block">
      <span class="logo-text">FH6</span>
      <span class="logo-sub">FARM BOT</span>
    </div>
    <div class="logo-sep"></div>
    <StatusPill {state} {subState} />
  </div>

  <!-- Right: Stats -->
  <div class="stats">
    <div class="stat">
      <span class="stat-label">CYCLE</span>
      <span class="stat-value mono">{cycle}</span>
    </div>
    <div class="stat-sep"></div>
    <div class="stat">
      <span class="stat-label">SKILL PTS</span>
      <span class="stat-value sp">{sessionSp}</span>
    </div>
    <div class="stat-sep"></div>
    <div class="stat">
      <span class="stat-label">CREDITS</span>
      <span class="stat-value cr">{formatCr(sessionCr)}</span>
    </div>
    <div class="stat-sep"></div>
    <div class="stat">
      <span class="stat-label">WHEELSPINS</span>
      <span class="stat-value ws">{sessionWspins}</span>
    </div>
  </div>
  {#if state === "idle"}
    <div class="header-right">
      <button
        class="settings-btn"
        on:click={() => dispatch("settings")}
        title="Settings"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="gear-icon"
        >
          <circle cx="12" cy="12" r="3"></circle>
          <path
            d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
          ></path>
        </svg>
      </button>
    </div>
  {/if}
</header>

<style>
  .bot-header {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0;
    padding: 0 12px;
    height: 64px;
    flex-shrink: 0;
    background: #1d2a2c;
    border-bottom: none;
    overflow: hidden;
    z-index: 10;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.2);
  }

  /* No diagonal sweep */

  /* ── Left ─────────────────────────────────────────────────── */
  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 0 1 auto;
    min-width: 0;
    position: relative;
    z-index: 1;
    margin-right: 12px;
  }

  .logo-block {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .logo-text {
    font-family: var(--font-display);
    font-size: 26px;
    font-weight: 900;
    letter-spacing: 0.04em;
    color: var(--fh6-lime);
    line-height: 1;
  }

  .logo-sub {
    font-size: 8px;
    font-weight: 700;
    letter-spacing: 0.22em;
    color: #ffffff;
    text-transform: uppercase;
    line-height: 1;
    margin-top: 2px;
  }

  .logo-sep {
    width: 1px;
    height: 32px;
    background: rgba(255, 255, 255, 0.12);
    flex-shrink: 0;
  }

  /* ── Stats ──────────────────────────────────────────── */
  .stats {
    display: flex;
    align-items: center;
    gap: 0;
    margin-left: auto;
    flex-shrink: 0;
    position: relative;
    z-index: 1;
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
    padding: 0 8px;
  }

  .stat-sep {
    width: 1px;
    height: 28px;
    background: rgba(255, 255, 255, 0.12);
    flex-shrink: 0;
  }

  .stat-label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.18em;
    color: rgba(255, 255, 255, 0.45);
    text-transform: uppercase;
    white-space: nowrap;
  }

  .stat-value {
    font-family: var(--font-mono);
    font-size: 28px;
    font-weight: 700;
    color: rgba(255, 255, 255, 0.95);
    line-height: 1;
    letter-spacing: -0.02em;
  }

  .stat-value.sp {
    color: var(--fh6-teal);
  }
  .stat-value.cr {
    color: #ffcc00;
  }
  .stat-value.ws {
    color: #a8d400;
  }

  .header-right {
    flex-shrink: 0;
    position: relative;
    z-index: 1;
    margin-left: 10px;
  }

  .settings-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    color: rgba(255, 255, 255, 0.65);
    cursor: pointer;
    transition: all 0.3s ease;
  }

  .settings-btn:hover {
    color: var(--fh6-lime);
    border-color: var(--fh6-lime-dim);
    background: rgba(0, 0, 0, 0.45);
    box-shadow: 0 0 8px rgba(202, 255, 2, 0.2);
  }

  .settings-btn:hover .gear-icon {
    transform: rotate(45deg);
  }

  .gear-icon {
    width: 24px;
    height: 24px;
    flex-shrink: 0;
    transition: transform 0.5s cubic-bezier(0.4, 0, 0.2, 1);
  }
</style>
