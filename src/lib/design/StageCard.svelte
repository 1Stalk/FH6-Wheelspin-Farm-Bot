<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { StageMeta } from "$lib/mock";

  export let meta: StageMeta;
  export let enabled: boolean = true;
  export let active: boolean = false;
  export let disabled: boolean = false;
  export let executing: boolean = false;

  const dispatch = createEventDispatcher<{
    toggle: { key: "stage1" | "stage2" | "stage3" | "stage4" };
  }>();

  function toggle() {
    if (!disabled) dispatch("toggle", { key: meta.key });
  }

  $: variant = meta.id ?? 1;

  const videoSrcMap = {
    stage1: "/assets/collosus.mp4",
    stage2: "/assets/sp_farm.mp4",
    stage3: "/assets/buy_cars.mp4",
    stage4: "/assets/spend_sp.mp4",
  };

  let videoEl: HTMLVideoElement;

  $: if (videoEl) {
    if (enabled) {
      videoEl.play().catch((err) => {
        console.warn("Video play interrupted or failed:", err);
      });
    } else {
      videoEl.pause();
    }
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
  class="card"
  class:enabled
  class:active
  class:disabled
  class:executing
  style="--c: {meta.color};"
  on:click={toggle}
>
  <!-- Stage image area (animated placeholder) -->
  <div class="card-visual variant-{variant}" class:enabled>
    {#if videoSrcMap[meta.key]}
      <!-- svelte-ignore a11y-media-has-caption -->
      <video
        bind:this={videoEl}
        src={videoSrcMap[meta.key]}
        loop
        muted
        playsinline
        autoplay={enabled}
      ></video>
    {/if}
  </div>

  <!-- Scanlines -->
  <div class="scanlines"></div>

  <!-- Stage number badge (top-left, filled with stage color) -->
  <div class="stage-badge">
    <span>S{meta.id}</span>
  </div>

  <!-- Active top bar -->
  {#if active}
    <div class="active-bar"></div>
  {/if}

  <!-- Bottom info bar (white on light theme) -->
  <div class="info-bar">
    <div class="info-left">
      <span class="stage-num">STAGE {meta.id}</span>
      <span class="stage-name">{meta.label}</span>
      <span class="stage-sub">{meta.sublabel}</span>
    </div>
    <button
      class="toggle-btn"
      class:on={enabled}
      {disabled}
      on:click|stopPropagation={toggle}
      title={enabled ? "Disable stage" : "Enable stage"}
    >
      <span class="toggle-dot" class:on={enabled}></span>
      {enabled ? "ON" : "OFF"}
    </button>
  </div>

  <!-- Disabled overlay -->
  {#if !enabled}
    <div class="disabled-veil">
      <span class="disabled-text">DISABLED</span>
    </div>
  {/if}
</div>

<style>
  .card {
    position: relative;
    aspect-ratio: 16 / 9;
    width: 100%;
    overflow: hidden;
    background: var(--bg-card);
    border-radius: 16px;
    /* Double border: inner #000, outer #caff02 */
    border: 4px solid #000000;
    box-shadow:
      0 0 0 3px #caff02,
      0 4px 12px rgba(0, 0, 0, 0.1);
    cursor: pointer;
    transition:
      box-shadow 0.25s,
      transform 0.2s;
  }

  .card.enabled:hover,
  .card.enabled.executing {
    box-shadow:
      0 0 0 4px #caff02,
      0 0 0 6px rgba(202, 255, 2, 0.35),
      0 8px 24px rgba(0, 0, 0, 0.16);
    transform: translateY(-2px) scale(1.01);
    z-index: 2;
  }

  .card.active {
    box-shadow:
      0 0 0 4px #caff02,
      0 0 0 8px rgba(202, 255, 2, 0.3),
      0 4px 20px rgba(0, 0, 0, 0.12);
  }

  .card.disabled {
    cursor: not-allowed;
  }

  /* ── Visual area ──────────────────────────────────────────── */
  .card-visual {
    position: absolute;
    inset: 0;
    overflow: hidden;
    filter: grayscale(1) brightness(1.1) saturate(0);
    transition: filter 0.6s ease;
  }

  .card-visual.enabled {
    filter: grayscale(0) brightness(1) saturate(1);
  }

  .card-visual video {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    z-index: 1;
  }

  /* Background pattern per variant */
  .card-visual::before {
    content: "";
    position: absolute;
    inset: 0;
    background: radial-gradient(
      ellipse 95% 80% at 50% 50%,
      color-mix(in srgb, var(--c) 18%, transparent) 0%,
      color-mix(in srgb, var(--c) 5%, var(--bg-card)) 60%,
      var(--bg-card) 100%
    );
    z-index: 2;
  }

  .card-visual::after {
    content: "";
    position: absolute;
    inset: 0;
    opacity: 0.5;
    z-index: 3;
  }

  /* Variant 1 — diagonal speed lines */
  .variant-1::after {
    background: repeating-linear-gradient(
      -8deg,
      transparent 0px,
      transparent 22px,
      color-mix(in srgb, var(--c) 20%, transparent) 22px,
      color-mix(in srgb, var(--c) 20%, transparent) 24px
    );
  }

  /* Variant 2 — conic diamond */
  .variant-2::after {
    background: repeating-conic-gradient(
      from 45deg at 50% 50%,
      color-mix(in srgb, var(--c) 12%, transparent) 0deg,
      transparent 12deg,
      transparent 78deg,
      color-mix(in srgb, var(--c) 12%, transparent) 90deg
    );
  }

  /* Variant 3 — grid */
  .variant-3::after {
    background: linear-gradient(rgba(0, 0, 0, 0.08) 1px, transparent 1px),
      linear-gradient(90deg, rgba(0, 0, 0, 0.08) 1px, transparent 1px);
    background-size: 18px 18px;
  }

  /* Variant 4 — radial rings */
  .variant-4::after {
    background: repeating-radial-gradient(
      circle at 50% 50%,
      transparent 0px,
      transparent 20px,
      color-mix(in srgb, var(--c) 10%, transparent) 20px,
      color-mix(in srgb, var(--c) 10%, transparent) 22px
    );
  }

  /* ── Scanlines ────────────────────────────────────────────── */
  .scanlines {
    position: absolute;
    inset: 0;
    background: repeating-linear-gradient(
      0deg,
      transparent 0px,
      transparent 3px,
      rgba(255, 255, 255, 0.18) 3px,
      rgba(255, 255, 255, 0.18) 4px
    );
    pointer-events: none;
    z-index: 1;
  }

  /* ── Stage badge (top-left) ───────────────────────────── */
  .stage-badge {
    position: absolute;
    top: 8px;
    left: 8px;
    background: #caff02;
    padding: 3px 10px;
    border-radius: 4px;
    border: 1.5px solid #000000;
    z-index: 4;
  }

  .stage-badge span {
    font-family: var(--font-display);
    font-size: 11px;
    font-weight: 900;
    letter-spacing: 0.08em;
    color: #1a2000;
    line-height: 1;
  }

  /* ── Active top bar ──────────────────────────────────────── */
  .active-bar {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: var(--c);
    z-index: 5;
    animation: bar-slide 2s ease-in-out infinite;
  }

  @keyframes bar-slide {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  /* ── Info bar (bottom) ───────────────────────────────────── */
  .info-bar {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: linear-gradient(
      to top,
      rgba(255, 255, 255, 0.97) 0%,
      rgba(255, 255, 255, 0.88) 60%,
      transparent 100%
    );
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    padding: 8px 12px 9px;
    z-index: 3;
  }

  .info-left {
    display: flex;
    flex-direction: column;
    gap: 1px;
    flex: 1;
    min-width: 0;
  }

  .stage-num {
    font-size: 8px;
    font-weight: 800;
    letter-spacing: 0.25em;
    color: var(--c);
    /* make sure lime is readable: darken it a bit */
    filter: brightness(0.75);
    text-transform: uppercase;
    line-height: 1;
  }

  .stage-name {
    font-family: var(--font-display);
    font-size: 16px;
    font-weight: 900;
    letter-spacing: 0.06em;
    color: var(--text-primary);
    text-transform: uppercase;
    line-height: 1.05;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .stage-sub {
    font-size: 9px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    line-height: 1;
  }

  /* ── Toggle button ───────────────────────────────────────── */
  .toggle-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    font-size: 10px;
    font-weight: 800;
    letter-spacing: 0.18em;
    background: rgba(255, 255, 255, 0.85);
    border: 1.5px solid var(--border-mid);
    color: var(--text-muted);
    clip-path: var(--clip-sm);
    transition:
      border-color 0.2s,
      color 0.2s,
      box-shadow 0.2s,
      background 0.2s;
    cursor: pointer;
    flex-shrink: 0;
  }

  .toggle-btn.on {
    border-color: var(--c);
    color: var(--text-primary);
    background: white;
  }

  .toggle-btn.on:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.14);
  }

  .toggle-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text-dim);
    flex-shrink: 0;
    transition: background 0.2s;
  }

  .toggle-dot.on {
    background: var(--c);
    box-shadow: 0 0 4px var(--c);
    animation: dot-pulse 1.5s ease-in-out infinite;
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

  .disabled-veil {
    position: absolute;
    inset: 0;
    background: rgba(233, 233, 233, 0.75);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 5;
    backdrop-filter: grayscale(1);
    pointer-events: none;
  }

  .disabled-text {
    font-family: var(--font-display);
    font-size: 12px;
    font-weight: 900;
    letter-spacing: 0.35em;
    color: var(--text-muted);
    text-transform: uppercase;
    border: 1.5px solid var(--border-mid);
    padding: 4px 14px;
    background: white;
    clip-path: var(--clip-sm);
  }

  /* Corners and cut-corners removed (rounded corners replace them) */
</style>
