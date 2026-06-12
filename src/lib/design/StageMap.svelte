<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import StageCard from "./StageCard.svelte";
  import { STAGE_META } from "$lib/mock";

  export let stagesEnabled: {
    stage1: boolean;
    stage2: boolean;
    stage3: boolean;
    stage4: boolean;
  } = {
    stage1: true,
    stage2: true,
    stage3: true,
    stage4: true,
  };
  export let botState: string = "idle";
  export let subState: string | null = null;
  export let disabled: boolean = false;
  const dispatch = createEventDispatcher<{
    toggle: { key: 'stage1' | 'stage2' | 'stage3' | 'stage4' };
    choose_car: { stage: 'stage1' | 'stage2' };
  }>();

  // Arrow is active only when navigating between stages (nav modules)
  $: activeArrowKey = (botState.startsWith("nav_") ? botState.slice(4) : null) ||
                      (subState === "Navigation → Colossus" ? "nav_to_stage1" :
                       subState === "Navigation → Custom Map" ? "nav_to_stage2" :
                       subState === "Navigation → Car Collection" ? "nav_to_stage3" :
                       subState === "Navigation → Design & Paint" ? "nav_to_stage4" : null);

  const ARROW_SOURCE_MAP: Record<string, string> = {
    nav_to_stage2: "stage1", // Navigating from S1 to S2 -> Arrow S1 -> S2 (from stage1)
    nav_to_stage3: "stage2", // Navigating from S2 to S3 -> Arrow S2 -> S3 (from stage2)
    nav_to_stage4: "stage3", // Navigating from S3 to S4 -> Arrow S3 -> S4 (from stage3)
    nav_to_stage1: "stage4", // Navigating from S4 to S1 -> Arrow S4 -> S1 (from stage4)
  };

  $: activeKey = activeArrowKey ? (ARROW_SOURCE_MAP[activeArrowKey] ?? null) : null;
  $: executingStage = botState.startsWith("stage_") ? "stage" + botState.slice(6) : null;

  function onToggle(e: CustomEvent<{ key: 'stage1' | 'stage2' | 'stage3' | 'stage4' }>) {
    dispatch("toggle", e.detail);
  }

  // Grid dims: 2×260px cols + 18px gap = 538px wide
  // Card height: 260×9/16 ≈ 146px; 2 rows + 12px gap = 304px tall
  // Arrows: extend 50px inside each card
  const ARROWS = [
    // S1→S2: horizontal top (y=73 = card center), x crosses gap at 260..278
    { id: "a12", from: "stage1", x1: 210, y1: 73, x2: 328, y2: 73 },
    // S2→S3: vertical right (x=408 = right card center), y crosses gap at 146..158
    { id: "a23", from: "stage2", x1: 408, y1: 126, x2: 408, y2: 178 },
    // S3→S4: horizontal bottom (y=231), going right→left
    { id: "a34", from: "stage3", x1: 328, y1: 231, x2: 210, y2: 231 },
    // S4→S1: vertical left (x=130 = left card center), going bottom→top
    { id: "a41", from: "stage4", x1: 130, y1: 178, x2: 130, y2: 126 },
  ];

  // ── Mouse tracking for interactive background ──
  let mx: number = 50;
  let my: number = 50;
  let hovering: boolean = false;

  function onMouseMove(e: MouseEvent & { currentTarget: EventTarget & HTMLDivElement }) {
    const rect = e.currentTarget.getBoundingClientRect();
    mx = ((e.clientX - rect.left) / rect.width) * 100;
    my = ((e.clientY - rect.top) / rect.height) * 100;
    hovering = true;
  }

  function onMouseLeave() {
    hovering = false;
  }
</script>

<div
  role="presentation"
  class="stage-map"
  on:mousemove={onMouseMove}
  on:mouseleave={onMouseLeave}
  style="--mx: {mx}%; --my: {my}%;"
>
  <!-- Interactive background -->
  <div class="map-bg" class:hovered={hovering}></div>

  <!-- Grid + arrows wrapped together so SVG aligns exactly with cards -->
  <div class="grid-container">
    <div class="grid">
      {#each STAGE_META as meta (meta.key)}
        {@const isExecuting = executingStage === meta.key}
        <div 
          class="card-wrapper" 
          class:stage1={meta.key === 'stage1'} 
          class:stage2={meta.key === 'stage2'}
          class:disabled={!stagesEnabled[meta.key]}
          class:executing={isExecuting}
        >
          {#if meta.key === 'stage1'}
            <button
              class="choose-car-tab tab-s1"
              title="Choose Car for Stage 1"
              on:click|stopPropagation={() => dispatch("choose_car", { stage: "stage1" })}
            >
              <span class="tab-icon">🏎️</span>
              <span class="tab-label">Choose Car</span>
            </button>
          {:else if meta.key === 'stage2'}
            <button
              class="choose-car-tab tab-s2"
              title="Choose Car for Stage 2"
              on:click|stopPropagation={() => dispatch("choose_car", { stage: "stage2" })}
            >
              <span class="tab-icon">🏎️</span>
              <span class="tab-label">Choose Car</span>
            </button>
          {/if}
          <StageCard
            {meta}
            enabled={stagesEnabled[meta.key]}
            active={activeKey === meta.key}
            {disabled}
            executing={isExecuting}
            on:toggle={onToggle}
          />
        </div>
      {/each}
    </div>

    <svg
      class="arrows-svg"
      viewBox="0 0 538 304"
      preserveAspectRatio="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <defs>
        <marker
          id="arr-idle"
          markerWidth="7"
          markerHeight="7"
          refX="5"
          refY="3.5"
          orient="auto"
        >
          <path d="M0,0 L0,7 L6,3.5 Z" fill="rgba(0,0,0,0.18)" />
        </marker>
        <marker
          id="arr-active"
          markerWidth="7"
          markerHeight="7"
          refX="5"
          refY="3.5"
          orient="auto"
        >
          <path d="M0,0 L0,7 L6,3.5 Z" fill="#a8d400" />
        </marker>
        <filter id="glow-lime" x="-60%" y="-60%" width="220%" height="220%">
          <feGaussianBlur in="SourceGraphic" stdDeviation="2.5" result="blur" />
          <feMerge
            ><feMergeNode in="blur" /><feMergeNode
              in="SourceGraphic"
            /></feMerge
          >
        </filter>
      </defs>

      {#each ARROWS as arrow}
        {@const isActive = activeKey === arrow.from}
        {@const dx = arrow.x2 - arrow.x1}
        {@const dy = arrow.y2 - arrow.y1}
        {@const len = Math.hypot(dx, dy)}
        {@const animOffset = 8}
        {@const ax2 = arrow.x2 - (dx / len) * animOffset}
        {@const ay2 = arrow.y2 - (dy / len) * animOffset}
        <line
          class="arrow-line"
          class:active={isActive}
          x1={arrow.x1}
          y1={arrow.y1}
          x2={arrow.x2}
          y2={arrow.y2}
          stroke={isActive ? "#a8d400" : "rgba(0,0,0,0.22)"}
          stroke-width={isActive ? 2 : 1.2}
          stroke-dasharray={isActive ? "5 3" : "3 5"}
          marker-end={isActive ? "url(#arr-active)" : "url(#arr-idle)"}
        />
        {#if isActive}
          <circle r="3.5" fill="#a8d400" filter="url(#glow-lime)" opacity="0.9">
            <animateMotion
              dur="1.0s"
              repeatCount="indefinite"
              path="M{arrow.x1},{arrow.y1} L{ax2},{ay2}"
            />
          </circle>
        {/if}
      {/each}
    </svg>
  </div>
</div>

<style>
  .stage-map {
    position: relative;
    width: 100%;
    height: auto;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 36px 48px;
    box-sizing: border-box;
  }

  /* ── Interactive background ─────────────────────────────── */
  .map-bg {
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 0;
    /* Constant subtle tint — same in both states */
    background-color: rgba(0, 0, 0, 0.03);
    /* Dot grid */
    background-image: radial-gradient(
      circle,
      rgba(0, 0, 0, 0.13) 1px,
      transparent 1px
    );
    background-size: 22px 22px;
  }

  /* On hover: add lime spotlight on top, dot grid stays identical */
  .map-bg.hovered {
    background-image: radial-gradient(
        circle 200px at var(--mx) var(--my),
        rgba(202, 255, 2, 0.18) 0%,
        rgba(202, 255, 2, 0.06) 40%,
        transparent 70%
      ),
      radial-gradient(circle, rgba(0, 0, 0, 0.13) 1px, transparent 1px);
    background-size:
      auto,
      22px 22px;
  }

  /* Wrapper sized exactly to the grid — SVG aligns with cards */
  .grid-container {
    position: relative;
    flex-shrink: 0;
    z-index: 1;
  }

  .grid {
    display: grid;
    grid-template-columns: 230px 230px;
    grid-template-rows: auto auto;
    gap: 12px 16px;
  }

  .card-wrapper {
    position: relative;
    width: 100%;
    height: 100%;
  }

  .card-wrapper :global(.card) {
    position: relative;
    z-index: 2;
  }

  /* ── Choose Car Tab Button (Stage 1 & 2) ───────────── */
  .choose-car-tab {
    position: absolute;
    bottom: calc(100% - 4px); /* sits on top of the card top border */
    left: 20px; /* align with the left part of the card */
    z-index: 1; /* behind the card */
    border: 3px solid #000000;
    border-bottom: none;
    border-radius: 8px 8px 0 0;
    padding: 5px 12px 6px 12px;
    font-family: var(--font-display);
    font-size: 10px;
    font-weight: 900;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    cursor: pointer;
    outline: none;
    
    display: flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
    
    /* Transition when card gets hovered */
    transition: transform 0.2s ease, background-color 0.2s, box-shadow 0.2s;
  }

  .choose-car-tab.tab-s1 {
    left: 20px;
    background: #ffcc00;
    color: #1a2000;
    box-shadow: 0 -2px 6px rgba(0, 0, 0, 0.2);
  }

  .choose-car-tab.tab-s2 {
    right: 20px;
    left: auto;
    background: #2cbca4;
    color: #12191b;
    box-shadow: 0 -2px 6px rgba(0, 0, 0, 0.2);
  }

  .tab-icon {
    font-size: 11px;
    display: inline-block;
  }

  .tab-label {
    opacity: 1; /* always fully visible */
  }

  /* When the wrapper is hovered (mouse over the card block) and stage is enabled,
     the tab scales and translates up to match the card's hover behavior */
  .card-wrapper:not(.disabled):hover .choose-car-tab.tab-s1,
  .card-wrapper.executing:not(.disabled) .choose-car-tab.tab-s1 {
    transform: translateY(-2px) scale(1.01);
    background: #ffe066;
    box-shadow: 
      0 -2px 0 1.5px rgba(202, 255, 2, 0.35), /* match card hover outline glow */
      0 -4px 12px rgba(255, 204, 0, 0.3);
  }

  .card-wrapper:not(.disabled):hover .choose-car-tab.tab-s2,
  .card-wrapper.executing:not(.disabled) .choose-car-tab.tab-s2 {
    transform: translateY(-2px) scale(1.01);
    background: #4ad2be;
    box-shadow: 
      0 -2px 0 1.5px rgba(202, 255, 2, 0.35), /* match card hover outline glow */
      0 -4px 12px rgba(44, 188, 164, 0.3);
  }

  /* Style Choose Car tab when its stage is disabled */
  .card-wrapper.disabled .choose-car-tab {
    opacity: 0.3;
    cursor: not-allowed;
    pointer-events: none;
    box-shadow: none;
  }

  .arrows-svg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 10;
    overflow: visible;
  }

  .arrow-line.active {
    filter: drop-shadow(0 0 3px rgba(168, 212, 0, 0.5));
  }
</style>
