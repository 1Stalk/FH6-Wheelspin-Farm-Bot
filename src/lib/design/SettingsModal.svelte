<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { fade, scale } from "svelte/transition";
  import { invoke } from "@tauri-apps/api/core";

  export let connected: boolean = false;
  export let smartSettings: boolean = false;

  let localSmartSettings = smartSettings;
  let driverInstalled = false;
  let isChecking = true;

  $: localSmartSettings = smartSettings;

  const dispatch = createEventDispatcher<{
    close: null;
    save: { smartSettings: boolean };
    run_nav: { target: string };
    test_gamepad: null;
    test_cv: null;
    open_setup: null;
  }>();

  function close() {
    dispatch("close");
  }

  function save() {
    dispatch("save", { smartSettings: localSmartSettings });
    close();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") close();
  }

  function runNav(target: string) {
    dispatch("run_nav", { target });
  }

  function runTestGamepad() {
    dispatch("test_gamepad");
  }

  function runTestCV() {
    dispatch("test_cv");
  }

  function runSetup() {
    dispatch("open_setup");
  }

  async function checkDriverStatus() {
    isChecking = true;
    try {
      driverInstalled = await invoke<boolean>("check_vigem_status");
    } catch (err) {
      console.error("Failed to check ViGEmBus status:", err);
      driverInstalled = false;
    } finally {
      isChecking = false;
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    checkDriverStatus();
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

<div
  role="button"
  aria-label="Close modal"
  tabindex="-1"
  class="backdrop"
  transition:fade={{ duration: 180 }}
  on:click|self={close}
  on:keydown|self={(e) => (e.key === "Enter" || e.key === " ") && close()}
>
  <div class="modal" transition:scale={{ start: 0.96, duration: 180 }}>
    <!-- Header -->
    <div class="modal-header">
      <span class="modal-title">ADVANCED SETTINGS</span>
      <button class="close-btn" on:click={close} title="Close">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="close-icon"
        >
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>

    <!-- Body -->
    <div class="modal-body">
      <div class="status-section">
        <span class="status-label">VIRTUAL GAMEPAD CONNECTION</span>
        <div class="status-row">
          <div
            class="conn"
            class:online={driverInstalled}
            class:checking={isChecking}
          >
            <span class="conn-dot"></span>
            <div class="conn-text">
              <span class="conn-status">
                {#if isChecking}
                  CHECKING
                {:else if driverInstalled}
                  CONNECTED
                {:else}
                  DISCONNECTED
                {/if}
              </span>
              <span class="conn-label">VIGEMBUS DRIVER</span>
            </div>
          </div>

          <div class="status-buttons">
            <button
              class="setup-inline-btn"
              on:click={runTestGamepad}
              title="Test Inputs Diagnostics"
            >
              🎮 TEST INPUTS
            </button>

            <button
              class="setup-inline-btn"
              on:click={runSetup}
              title="Run Initial Setup / Driver Installation"
            >
              🛠️ INITIAL SETUP
            </button>
          </div>
        </div>
      </div>

      <!-- Test buttons section (Commented out per user request)
      <div class="settings-section">
        <div class="section-header">
          <span>TEST CONTROLS (NAV MODULES)</span>
        </div>
        <div class="nav-buttons-grid">
          <button
            class="nav-test-btn nav-s1"
            on:click={() => runNav("nav_to_stage1")}
            title="Test Nav to Stage 1">Nav to stage 1</button
          >
          <button
            class="nav-test-btn nav-s2"
            on:click={() => runNav("nav_to_stage2")}
            title="Test Nav to Stage 2">Nav to stage 2</button
          >
          <button
            class="nav-test-btn nav-s3"
            on:click={() => runNav("nav_to_stage3")}
            title="Test Nav to Stage 3">Nav to stage 3</button
          >
          <button
            class="nav-test-btn nav-s4"
            on:click={() => runNav("nav_to_stage4")}
            title="Test Nav to Stage 4">Nav to stage 4</button
          >
        </div>

        <div class="test-separator"></div>

        <div style="display: flex; gap: 12px;">
          <button
            class="test-inputs-btn"
            on:click={runTestGamepad}
            title="Test Inputs Diagnostics"
            style="flex: 1;"
          >
            <span>🎮 TEST INPUTS</span>
          </button>
          
          <button
            class="test-inputs-btn"
            on:click={runTestCV}
            title="Test CV Screen Diagnostics"
            style="flex: 1; border-color: rgba(44, 188, 164, 0.5); color: var(--fh6-teal);"
          >
            <span>🔍 TEST CV</span>
          </button>
        </div>

      </div>
      -->

      <!-- Smart Settings section -->
      <div class="settings-section">
        <div class="section-header">
          <span>SMART CONFIGURATION</span>
        </div>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">SMART SETTINGS</span>
            <span class="setting-desc"
              >Auto-adjusts stage parameters on change to prevent configuration conflicts.</span
            >
          </div>
          <label class="switch-container">
            <input
              type="checkbox"
              bind:checked={localSmartSettings}
              class="switch-input"
            />
            <div class="switch-track">
              <div class="switch-thumb"></div>
            </div>
          </label>
        </div>
      </div>

      <!-- Buy Pro Section -->
      <div class="settings-section pro-section">
        <div class="pro-section-content">
          <div class="pro-header-row">
            <span class="pro-tag">PRO VERSION</span>
            <span class="pro-desc">Unlock premium features and support development!</span>
          </div>
          <div class="pro-features">
            <span>★ Custom Cars (Stage 1 & 2)</span>
            <span>★ Discord & Telegram Integration</span>
            <span>★ Safe Auto-Updater</span>
          </div>
        </div>
        <a href="https://ko-fi.com/s/6559bf5583" target="_blank" rel="noopener noreferrer" class="pro-buy-btn">
          <svg class="kofi-icon" fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" width="16" height="16" style="margin-right: 2px; vertical-align: middle; position: relative; top: -1px;">
            <path d="M11.351 2.715c-2.7 0-4.986.025-6.83.26C2.078 3.285 0 5.154 0 8.61c0 3.506.182 6.13 1.585 8.493 1.584 2.701 4.233 4.182 7.662 4.182h.83c4.209 0 6.494-2.234 7.637-4a9.5 9.5 0 0 0 1.091-2.338C21.792 14.688 24 12.22 24 9.208v-.415c0-3.247-2.13-5.507-5.792-5.87-1.558-.156-2.65-.208-6.857-.208m0 1.947c4.208 0 5.09.052 6.571.182 2.624.311 4.13 1.584 4.13 4v.39c0 2.156-1.792 3.844-3.87 3.844h-.935l-.156.649c-.208 1.013-.597 1.818-1.039 2.546-.909 1.428-2.545 3.064-5.922 3.064h-.805c-2.571 0-4.831-.883-6.078-3.195-1.09-2-1.298-4.155-1.298-7.506 0-2.181.857-3.402 3.012-3.714 1.533-.233 3.559-.26 6.39-.26m6.547 2.287c-.416 0-.65.234-.65.546v2.935c0 .311.234.545.65.545 1.324 0 2.051-.754 2.051-2s-.727-2.026-2.052-2.026m-10.39.182c-1.818 0-3.013 1.48-3.013 3.142 0 1.533.858 2.857 1.949 3.897.727.701 1.87 1.429 2.649 1.896a1.47 1.47 0 0 0 1.507 0c.78-.467 1.922-1.195 2.623-1.896 1.117-1.039 1.974-2.364 1.974-3.897 0-1.662-1.247-3.142-3.039-3.142-1.065 0-1.792.545-2.338 1.298-.493-.753-1.246-1.298-2.312-1.298"/>
          </svg>
          SUPPORT & GET PRO ON KO-FI
        </a>
      </div>
    </div>

    <!-- Footer -->
    <div class="modal-footer">
      <button class="action-btn cancel" on:click={close}>Cancel</button>
      <button class="action-btn save" on:click={save}>Apply & Save</button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(18, 25, 27, 0.6);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    outline: none;
    cursor: default;
  }

  .modal {
    background: #1b2628;
    width: 580px;
    max-width: 90vw;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: 12px;
    /* Double outline matching StageCard */
    border: 4px solid #000000;
    box-shadow:
      0 0 0 3px var(--fh6-lime),
      0 16px 36px rgba(0, 0, 0, 0.5);
    cursor: default;
  }

  /* ── Header ─────────────────────────────────────────── */
  .modal-header {
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 2px solid var(--fh6-lime);
    background: rgba(0, 0, 0, 0.25);
    padding: 0 20px;
    flex-shrink: 0;
  }

  .modal-title {
    font-size: 14px;
    font-weight: 900;
    letter-spacing: 0.18em;
    color: #ffffff;
    text-transform: uppercase;
  }

  .close-btn {
    background: transparent;
    border: none;
    padding: 0;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: rgba(255, 255, 255, 0.4);
    cursor: pointer;
    transition:
      color 0.2s,
      transform 0.2s;
  }

  .close-btn:hover {
    color: var(--fh6-lime);
    transform: scale(1.1);
  }

  .close-icon {
    width: 18px;
    height: 18px;
  }

  /* ── Body ───────────────────────────────────────────── */
  .modal-body {
    padding: 36px 24px;
    background: radial-gradient(
      circle at top left,
      rgba(202, 255, 2, 0.03) 0%,
      transparent 70%
    );
    max-height: 75vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-height: 160px;
  }

  /* ── Footer ─────────────────────────────────────────── */
  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 14px 20px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(0, 0, 0, 0.25);
  }

  .action-btn {
    font-size: 13px;
    font-weight: 800;
    letter-spacing: 0.08em;
    border-radius: 4px;
    text-transform: uppercase;
    transition: all 0.2s ease;
    padding: 9px 20px;
    cursor: pointer;
  }

  .action-btn.cancel {
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: rgba(255, 255, 255, 0.7);
  }

  .action-btn.cancel:hover {
    background: rgba(255, 255, 255, 0.06);
    color: #ffffff;
    border-color: rgba(255, 255, 255, 0.35);
  }

  .action-btn.save {
    background: var(--fh6-lime);
    border: 1.5px solid var(--fh6-lime-dim);
    color: var(--fh6-lime-solid);
  }

  .action-btn.save:hover {
    background: var(--fh6-lime-dim);
    box-shadow: 0 0 12px var(--fh6-lime-glow);
  }

  /* ── Status Indicator ────────────────────────────────── */
  .status-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 24px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.08);
    padding: 16px;
    border-radius: 8px;
    align-items: flex-start;
    width: 100%;
  }

  .status-label {
    font-size: 13px;
    font-weight: 800;
    letter-spacing: 0.14em;
    color: var(--fh6-lime);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    padding-bottom: 8px;
    margin-bottom: 4px;
    text-transform: uppercase;
    width: 100%;
  }

  .status-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    gap: 16px;
    margin-top: 4px;
  }

  .status-buttons {
    display: flex;
    gap: 8px;
  }

  .setup-inline-btn {
    font-family: var(--font-display);
    font-size: 13px;
    font-weight: 800;
    padding: 7px 16px;
    background: rgba(0, 0, 0, 0.25);
    border: 1.5px solid rgba(202, 255, 2, 0.3);
    color: var(--fh6-lime);
    clip-path: var(--clip-btn);
    transition: all 0.2s ease;
    cursor: pointer;
  }

  .setup-inline-btn:hover {
    border-color: var(--fh6-lime);
    background: rgba(202, 255, 2, 0.08);
    box-shadow: 0 0 10px var(--fh6-lime-glow);
    color: #ffffff;
  }

  .conn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 16px 8px 12px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(255, 255, 255, 0.1);
    clip-path: var(--clip-sm);
  }

  .conn-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #ea4335;
    display: block;
    flex-shrink: 0;
    transition:
      background 0.3s,
      box-shadow 0.3s;
  }

  .conn.checking .conn-dot {
    background: #fbbc05;
    animation: dot-pulse 1.5s ease-in-out infinite;
  }

  .conn.online .conn-dot {
    background: var(--fh6-lime);
    box-shadow: 0 0 8px var(--fh6-lime-glow);
    animation: dot-pulse 2s ease-in-out infinite;
  }

  @keyframes dot-pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.4;
    }
  }

  .conn-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .conn-status {
    font-size: 14px;
    font-weight: 800;
    letter-spacing: 0.12em;
    color: #ea4335;
    transition: color 0.3s;
    line-height: 1;
  }

  .conn.checking .conn-status {
    color: #fbbc05;
  }

  .conn.online .conn-status {
    color: var(--fh6-lime);
  }

  .conn-label {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.14em;
    color: rgba(255, 255, 255, 0.25);
    line-height: 1;
    text-transform: uppercase;
  }

  /* ── Settings Sections ──────────────────────────────── */
  .settings-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.08);
    padding: 18px;
    border-radius: 8px;
    margin-bottom: 16px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 4px 0;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
  }

  .setting-label {
    font-size: 14px;
    font-weight: 800;
    color: #ffffff;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .setting-desc {
    font-size: 17px;
    color: rgba(255, 255, 255, 0.7);
    line-height: 1.45;
  }

  /* Premium Toggle Switch */
  .switch-container {
    position: relative;
    display: inline-block;
    width: 44px;
    height: 22px;
    cursor: pointer;
    flex-shrink: 0;
  }

  .switch-input {
    opacity: 0;
    width: 0;
    height: 0;
    position: absolute;
  }

  .switch-track {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 11px;
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .switch-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.7);
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
  }

  .switch-input:checked + .switch-track {
    background: var(--fh6-lime-dim);
    border-color: var(--fh6-lime);
    box-shadow: 0 0 10px var(--fh6-lime-glow);
  }

  .switch-input:checked + .switch-track .switch-thumb {
    transform: translateX(22px);
    background: #12191b;
    box-shadow: 0 0 4px rgba(0, 0, 0, 0.6);
  }

  .switch-container:hover .switch-track {
    border-color: rgba(255, 255, 255, 0.3);
  }

  .switch-container:hover .switch-input:checked + .switch-track {
    border-color: var(--fh6-lime);
    box-shadow: 0 0 14px var(--fh6-lime-glow);
  }

  .section-header {
    font-size: 13px;
    font-weight: 800;
    letter-spacing: 0.14em;
    color: var(--fh6-lime);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    padding-bottom: 8px;
    margin-bottom: 4px;
    text-transform: uppercase;
  }

  /* ── Nav Test Buttons ───────────────────────────────── */
  .nav-buttons-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
    margin-top: 4px;
  }

  .nav-test-btn {
    font-family: var(--font-display);
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    padding: 10px 0;
    border-radius: 4px;
    cursor: pointer;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: rgba(255, 255, 255, 0.7);
    transition: all 0.2s ease;
    outline: none;
    text-align: center;
  }

  .nav-test-btn.nav-s1:hover {
    border-color: var(--fh6-lime);
    color: var(--fh6-lime);
    box-shadow: 0 0 10px rgba(202, 255, 2, 0.15);
    background: rgba(202, 255, 2, 0.05);
  }

  .nav-test-btn.nav-s2:hover {
    border-color: var(--fh6-teal);
    color: var(--fh6-teal);
    box-shadow: 0 0 10px rgba(44, 188, 164, 0.15);
    background: rgba(44, 188, 164, 0.05);
  }

  .nav-test-btn.nav-s3:hover {
    border-color: var(--fh6-pink);
    color: var(--fh6-pink);
    box-shadow: 0 0 10px rgba(254, 2, 136, 0.15);
    background: rgba(254, 2, 136, 0.05);
  }

  .nav-test-btn.nav-s4:hover {
    border-color: #f0a84a;
    color: #f0a84a;
    box-shadow: 0 0 10px rgba(240, 168, 74, 0.15);
    background: rgba(240, 168, 74, 0.05);
  }

  /* ── Test Inputs Button ──────────────────────────────── */
  .test-separator {
    height: 1px;
    background: rgba(255, 255, 255, 0.08);
    margin: 8px 0;
  }

  .test-inputs-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-family: var(--font-display);
    font-size: 11px;
    font-weight: 900;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    padding: 10px 0;
    border-radius: 4px;
    cursor: pointer;
    background: rgba(0, 0, 0, 0.35);
    border: 1.5px dashed rgba(202, 255, 2, 0.3);
    color: var(--fh6-lime);
    transition: all 0.2s ease;
    outline: none;
    width: 100%;
  }

  .test-inputs-btn:hover {
    border-color: var(--fh6-lime);
    background: rgba(202, 255, 2, 0.08);
    box-shadow: 0 0 12px var(--fh6-lime-glow);
    color: #ffffff;
  }

  /* ── Pro Section in Settings ─────────────────────────── */
  .pro-section {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    background: linear-gradient(135deg, rgba(41, 171, 226, 0.12) 0%, rgba(0, 0, 0, 0.25) 100%);
    border: 1.5px solid rgba(41, 171, 226, 0.4);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .pro-section-content {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
  }

  .pro-header-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .pro-tag {
    font-family: var(--font-display);
    font-size: 12px;
    font-weight: 950;
    color: #ffffff;
    background: linear-gradient(135deg, #29abe2 0%, #00b9fe 100%);
    padding: 2px 8px;
    border-radius: 4px;
    letter-spacing: 0.1em;
    border: 1.5px solid #000;
    box-shadow: 0 0 8px rgba(41, 171, 226, 0.4);
    height: fit-content;
  }

  .pro-desc {
    font-size: 14px;
    font-weight: 800;
    color: #ffffff;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .pro-features {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 12px;
    font-size: 13px;
    color: rgba(255, 255, 255, 0.75);
  }

  .pro-buy-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    font-family: var(--font-display);
    font-size: 14px;
    font-weight: 900;
    letter-spacing: 0.08em;
    padding: 10px 20px;
    background: #29abe2;
    color: #ffffff;
    border: 1.5px solid #00b9fe;
    border-radius: 4px;
    cursor: pointer;
    text-decoration: none;
    transition: all 0.2s ease;
    clip-path: polygon(6px 0%, 100% 0%, calc(100% - 6px) 100%, 0% 100%);
    box-shadow: 0 0 10px rgba(41, 171, 226, 0.3);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .pro-buy-btn:hover {
    background: #00b9fe;
    box-shadow: 0 0 12px rgba(41, 171, 226, 0.6);
    transform: scale(1.02);
  }

  .pro-buy-btn:active {
    transform: scale(1);
  }
</style>
