<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { fade, scale } from "svelte/transition";

  const dispatch = createEventDispatcher<{
    close: null;
  }>();

  let hoveredImage: string | null = null;
  let hoveredAlt = "";

  function handleImageHover(src: string, alt: string) {
    hoveredImage = src;
    hoveredAlt = alt;
  }

  function handleImageLeave() {
    hoveredImage = null;
  }

  function handleClose() {
    dispatch("close");
  }
</script>

<div
  role="button"
  aria-label="Start Notice Modal"
  tabindex="-1"
  class="backdrop"
  transition:fade={{ duration: 180 }}
>
  <div class="modal" transition:scale={{ start: 0.96, duration: 180 }}>
    <!-- Header -->
    <div class="modal-header">
      <span class="modal-title">Required Starting State</span>
    </div>

    <!-- Body -->
    <div class="modal-body">
      <div class="instruction-card">
        <p class="main-notice">
          Bot must always be started while in the open world, inside the pause
          menu on the first tab as shown in the screenshot below.
        </p>
      </div>

      <div class="step-images-grid single">
        <div class="image-container">
          <img
            src="/assets/PauseMenuScreenshot.png"
            alt="Required Start Menu"
            class="step-image"
            on:mouseenter={() =>
              handleImageHover(
                "/assets/PauseMenuScreenshot.png",
                "Required Start Menu",
              )}
            on:mouseleave={handleImageLeave}
          />
          <div class="image-label">Pause Menu</div>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="modal-footer">
      <button class="action-btn got-it" on:click={handleClose}> Got it </button>
    </div>
  </div>

  {#if hoveredImage}
    <div class="zoom-overlay" transition:fade={{ duration: 150 }}>
      <div
        class="zoom-preview-container"
        transition:scale={{ start: 0.95, duration: 150 }}
      >
        <img src={hoveredImage} alt={hoveredAlt} class="zoom-preview-image" />
        <div class="zoom-preview-label">{hoveredAlt}</div>
      </div>
    </div>
  {/if}
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(18, 25, 27, 0.7);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    outline: none;
    cursor: default;
  }

  .modal {
    background: #1b2628;
    width: 540px;
    max-width: 92vw;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: 12px;
    border: 4px solid #000000;
    box-shadow:
      0 0 0 3px var(--fh6-lime),
      0 20px 48px rgba(0, 0, 0, 0.6);
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
    font-size: 16px;
    font-weight: 900;
    letter-spacing: 0.18em;
    color: #ffffff;
    text-transform: uppercase;
  }

  /* ── Body ───────────────────────────────────────────── */
  .modal-body {
    padding: 24px;
    background: radial-gradient(
      circle at top left,
      rgba(202, 255, 2, 0.03) 0%,
      transparent 70%
    );
    max-height: 80vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--gap-lg);
  }

  .instruction-card {
    background: rgba(0, 0, 0, 0.2);
    border: 1.5px solid rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    padding: 16px;
  }

  .main-notice {
    font-size: 16px;
    font-weight: 900;
    line-height: 1.5;
    color: #ffffff;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    text-align: left;
    border-left: 4px solid var(--fh6-lime);
    padding-left: 14px;
    margin: 4px 0;
  }

  /* ── Image grids with hover zoom ───────────────────── */
  .step-images-grid.single {
    display: grid;
    grid-template-columns: 1fr;
    max-width: 440px;
    margin: 8px auto 0 auto;
  }
  .image-container {
    position: relative;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(0, 0, 0, 0.25);
  }
  .step-image {
    width: 100%;
    aspect-ratio: 16 / 9;
    object-fit: cover;
    display: block;
    border-radius: 5px;
    border: 1px solid transparent;
    transition:
      transform 0.2s cubic-bezier(0.25, 0.8, 0.25, 1),
      border-color 0.2s ease,
      box-shadow 0.2s ease;
    position: relative;
    z-index: 1;
    cursor: zoom-in;
  }
  .step-image:hover {
    transform: scale(1.02);
    border-color: var(--fh6-lime);
    box-shadow: 0 0 10px var(--fh6-lime-glow);
    z-index: 2;
  }
  .image-label {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: rgba(18, 25, 27, 0.85);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    color: rgba(255, 255, 255, 0.85);
    font-size: 12px;
    font-weight: 800;
    text-transform: uppercase;
    text-align: center;
    padding: 4px 0;
    pointer-events: none;
    letter-spacing: 0.05em;
    border-bottom-left-radius: 5px;
    border-bottom-right-radius: 5px;
  }

  /* ── Zoom Overlay ────────── */
  .zoom-overlay {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
    z-index: 10005;
  }
  .zoom-preview-container {
    position: relative;
    background: #12191b;
    border: 3px solid var(--fh6-lime);
    border-radius: 12px;
    box-shadow:
      0 20px 50px rgba(0, 0, 0, 0.85),
      0 0 30px var(--fh6-lime-glow);
    padding: 6px;
    padding-bottom: 40px;
    max-width: 85vw;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }
  .zoom-preview-image {
    max-width: 100%;
    max-height: calc(80vh - 65px);
    object-fit: contain;
    border-radius: 6px;
    display: block;
  }
  .zoom-preview-label {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: rgba(18, 25, 27, 0.9);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    border-top: 1.5px solid rgba(255, 255, 255, 0.08);
    color: var(--fh6-lime);
    font-size: 13px;
    font-weight: 800;
    text-transform: uppercase;
    text-align: center;
    padding: 8px 0;
    pointer-events: none;
    letter-spacing: 0.08em;
    border-bottom-left-radius: 9px;
    border-bottom-right-radius: 9px;
  }

  /* ── Footer ─────────────────────────────────────────── */
  .modal-footer {
    display: flex;
    justify-content: flex-end;
    padding: 14px 20px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(0, 0, 0, 0.25);
  }

  .action-btn {
    font-size: 13px;
    font-weight: 900;
    letter-spacing: 0.08em;
    border-radius: 4px;
    text-transform: uppercase;
    transition: all 0.2s ease;
    padding: 8px 18px;
    cursor: pointer;
  }

  .action-btn.got-it {
    background: var(--fh6-lime);
    border: 1.5px solid var(--fh6-lime-dim);
    color: var(--fh6-lime-solid);
  }

  .action-btn.got-it:hover {
    background: var(--fh6-lime-dim);
    box-shadow: 0 0 12px var(--fh6-lime-glow);
  }
</style>
