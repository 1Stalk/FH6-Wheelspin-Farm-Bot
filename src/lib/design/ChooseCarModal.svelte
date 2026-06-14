<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { fade, scale } from "svelte/transition";

  export let stage: "stage1" | "stage2" = "stage1";
  export let selectedCarId: string = "";

  // Set local state based on props
  let localCarId = selectedCarId;

  const dispatch = createEventDispatcher<{
    close: null;
    select: { carId: string };
  }>();

  interface CarOption {
    id: string;
    name: string;
    year: string;
    classType: "R" | "S2" | "pro";
    rating: string;
    desc: string;
    stats: {
      speed: number;
      handling: number;
      accel: number;
    };
    locked?: boolean;
  }

  // Tacoma & S-Cargo for Stage 1
  const STAGE_1_CARS: CarOption[] = [
    {
      id: "toyota_tacoma_fe",
      name: "Toyota Tacoma TRD Pro FE",
      year: "2019",
      classType: "R",
      rating: "998",
      desc: "Option A from Initial Setup. Tuning share code: 155 494 373. ~6m AFK on Collosus.",
      stats: { speed: 7.5, handling: 7.2, accel: 7.9 },
    },
    {
      id: "nissan_scargo_fe",
      name: "Nissan S-Cargo FE",
      year: "1989",
      classType: "R",
      rating: "998",
      desc: "Option B from Initial Setup. Tuning share code: 101 759 352. ~5:33m AFK on Collosus.",
      stats: { speed: 8.0, handling: 8.8, accel: 8.2 },
    },
    {
      id: "custom_pro",
      name: "Custom Car",
      year: "N/A",
      classType: "pro",
      rating: "PRO",
      desc: "Use any vehicle from your garage.",
      stats: { speed: 0, handling: 0, accel: 0 },
      locked: true,
    },
  ];

  // Subaru 22B for Stage 2
  const STAGE_2_CARS: CarOption[] = [
    {
      id: "subaru_impreza_22b",
      name: "Subaru Impreza 22B-STi",
      year: "1998",
      classType: "S2",
      rating: "899",
      desc: "Step 2 from Initial Setup. Tuning share code: 871 988 972. Essential for Skill Points farming.",
      stats: { speed: 8.5, handling: 8.9, accel: 9.0 },
    },
    {
      id: "custom_pro",
      name: "Custom Car",
      year: "N/A",
      classType: "pro",
      rating: "PRO",
      desc: "Use any vehicle from your garage.",
      stats: { speed: 0, handling: 0, accel: 0 },
      locked: true,
    },
  ];

  $: cars = stage === "stage1" ? STAGE_1_CARS : STAGE_2_CARS;

  // Sync prop changes if any
  $: if (selectedCarId) {
    localCarId = selectedCarId;
  }

  function selectCar(car: CarOption) {
    if (car.locked) return;
    localCarId = car.id;
  }

  function close() {
    dispatch("close");
  }

  function save() {
    dispatch("select", {
      carId: localCarId,
    });
    close();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") close();
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
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
  <div
    class="modal"
    transition:scale={{ start: 0.96, duration: 180 }}
    style="--theme: {stage === 'stage1' ? '#ffcc00' : '#2cbca4'}"
  >
    <!-- Header -->
    <div class="modal-header">
      <span class="modal-title"
        >CHOOSE CAR FOR STAGE {stage === "stage1" ? "1" : "2"}</span
      >
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
      <span class="section-label">SELECT VEHICLE</span>

      <div class="cars-list">
        {#each cars as car}
          <!-- svelte-ignore a11y-click-events-have-key-events -->
          <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
          <div
            class="car-card"
            class:selected={localCarId === car.id}
            class:locked={car.locked}
            on:click={() => selectCar(car)}
          >
            <div class="car-info">
              <div class="car-title-row">
                <span class="car-name">
                  {#if car.locked}🔒
                  {/if}{car.name}
                </span>
                {#if car.year !== "N/A"}
                  <span class="car-year">{car.year}</span>
                {/if}
              </div>
              <p class="car-desc">{car.desc}</p>

              {#if car.id !== "custom_pro"}
                <div class="stats-row">
                  <div class="stat">
                    <span class="stat-lbl">SPD</span>
                    <div class="stat-bar-container">
                      <div
                        class="stat-bar"
                        style="width: {car.stats.speed * 10}%;"
                      ></div>
                    </div>
                    <span class="stat-val">{car.stats.speed}</span>
                  </div>
                  <div class="stat">
                    <span class="stat-lbl">HND</span>
                    <div class="stat-bar-container">
                      <div
                        class="stat-bar"
                        style="width: {car.stats.handling * 10}%;"
                      ></div>
                    </div>
                    <span class="stat-val">{car.stats.handling}</span>
                  </div>
                  <div class="stat">
                    <span class="stat-lbl">ACC</span>
                    <div class="stat-bar-container">
                      <div
                        class="stat-bar"
                        style="width: {car.stats.accel * 10}%;"
                      ></div>
                    </div>
                    <span class="stat-val">{car.stats.accel}</span>
                  </div>
                </div>
              {:else}
                <div class="pro-locked-notice">
                  <span class="pro-notice-title">🔒 PRO FEATURE LOCKED</span>
                  <span class="pro-notice-subtitle"
                    >Custom Car option is only available in the Pro version.</span
                  >
                  <a
                    href="https://ko-fi.com/s/6559bf5583"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="pro-kofi-banner-link"
                    on:click|stopPropagation
                  >
                    <svg class="kofi-icon" fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" width="16" height="16" style="margin-right: 2px; vertical-align: middle; position: relative; top: -1px;">
                      <path d="M11.351 2.715c-2.7 0-4.986.025-6.83.26C2.078 3.285 0 5.154 0 8.61c0 3.506.182 6.13 1.585 8.493 1.584 2.701 4.233 4.182 7.662 4.182h.83c4.209 0 6.494-2.234 7.637-4a9.5 9.5 0 0 0 1.091-2.338C21.792 14.688 24 12.22 24 9.208v-.415c0-3.247-2.13-5.507-5.792-5.87-1.558-.156-2.65-.208-6.857-.208m0 1.947c4.208 0 5.09.052 6.571.182 2.624.311 4.13 1.584 4.13 4v.39c0 2.156-1.792 3.844-3.87 3.844h-.935l-.156.649c-.208 1.013-.597 1.818-1.039 2.546-.909 1.428-2.545 3.064-5.922 3.064h-.805c-2.571 0-4.831-.883-6.078-3.195-1.09-2-1.298-4.155-1.298-7.506 0-2.181.857-3.402 3.012-3.714 1.533-.233 3.559-.26 6.39-.26m6.547 2.287c-.416 0-.65.234-.65.546v2.935c0 .311.234.545.65.545 1.324 0 2.051-.754 2.051-2s-.727-2.026-2.052-2.026m-10.39.182c-1.818 0-3.013 1.48-3.013 3.142 0 1.533.858 2.857 1.949 3.897.727.701 1.87 1.429 2.649 1.896a1.47 1.47 0 0 0 1.507 0c.78-.467 1.922-1.195 2.623-1.896 1.117-1.039 1.974-2.364 1.974-3.897 0-1.662-1.247-3.142-3.039-3.142-1.065 0-1.792.545-2.338 1.298-.493-.753-1.246-1.298-2.312-1.298"/>
                    </svg>
                    Support & Get Pro version on Ko-Fi
                  </a>
                </div>
              {/if}
            </div>

            <!-- Authentic Forza badge -->
            <div class="class-badge-container">
              <div class="class-badge badge-{car.classType.toLowerCase()}">
                <span class="badge-letter">{car.classType}</span>
                <span class="badge-rating">{car.rating}</span>
              </div>

              {#if !car.locked}
                <div class="radio-indicator">
                  <div class="radio-dot"></div>
                </div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- Footer -->
    <div class="modal-footer">
      <button class="action-btn cancel" on:click={close}>Cancel</button>
      <button class="action-btn save" on:click={save}>Select & Apply</button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(18, 25, 27, 0.65);
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
    width: 600px;
    max-width: 92vw;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: 12px;
    border: 4px solid #000000;
    box-shadow:
      0 0 0 3px var(--theme),
      0 16px 36px rgba(0, 0, 0, 0.5);
    cursor: default;
    transition: box-shadow 0.3s;
  }

  /* ── Header ─────────────────────────────────────────── */
  .modal-header {
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 2px solid var(--theme);
    background: rgba(0, 0, 0, 0.25);
    padding: 0 20px;
    flex-shrink: 0;
    transition: border-color 0.3s;
  }

  .modal-title {
    font-size: 17px;
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
    color: var(--theme);
    transform: scale(1.1);
  }

  .close-icon {
    width: 18px;
    height: 18px;
  }

  /* ── Body ───────────────────────────────────────────── */
  .modal-body {
    padding: 24px;
    background: radial-gradient(
      circle at top left,
      rgba(255, 255, 255, 0.015) 0%,
      transparent 70%
    );
    max-height: 70vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .section-label {
    font-size: 13px;
    font-weight: 800;
    letter-spacing: 0.14em;
    color: var(--theme);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    padding-bottom: 8px;
    text-transform: uppercase;
    width: 100%;
    transition: color 0.3s;
  }

  .cars-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-top: 4px;
  }

  .car-card {
    background: rgba(0, 0, 0, 0.25);
    border: 1.5px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 14px 16px;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
    cursor: pointer;
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .car-card:hover:not(.locked) {
    background: rgba(255, 255, 255, 0.03);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .car-card.selected {
    background: rgba(255, 255, 255, 0.02);
    border-color: var(--theme);
    box-shadow: 0 0 12px rgba(255, 255, 255, 0.05);
  }

  .car-card.locked {
    cursor: default;
    border-style: dashed;
    border-color: rgba(255, 255, 255, 0.12);
  }

  .car-card.locked:hover {
    background: rgba(41, 171, 226, 0.02);
    border-color: rgba(41, 171, 226, 0.3);
  }

  .car-card.locked .car-name,
  .car-card.locked .car-year,
  .car-card.locked .class-badge-container {
    opacity: 0.65;
  }

  .car-card.locked .car-desc {
    color: rgba(255, 255, 255, 0.7);
  }

  .car-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
    min-width: 0;
  }

  .car-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .car-name {
    font-size: 16px;
    font-weight: 800;
    color: #ffffff;
    letter-spacing: 0.04em;
  }

  .car-year {
    font-size: 15px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.35);
    background: rgba(255, 255, 255, 0.08);
    padding: 2px 8px;
    border-radius: 4px;
  }

  .car-desc {
    font-size: 16px;
    color: rgba(255, 255, 255, 0.7);
    line-height: 1.5;
    margin: 2px 0 6px 0;
  }

  /* Stat progress bars */
  .stats-row {
    display: flex;
    gap: 18px;
    margin-top: 4px;
  }

  .stat {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: var(--font-mono);
  }

  .stat-lbl {
    font-size: 15px;
    font-weight: 700;
    color: rgba(255, 255, 255, 0.35);
  }

  .stat-bar-container {
    width: 60px;
    height: 6px;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 3px;
    overflow: hidden;
  }

  .stat-bar {
    height: 100%;
    background: var(--theme);
    border-radius: 3px;
    transition: background-color 0.3s;
  }

  .stat-val {
    font-size: 16px;
    font-weight: 700;
    color: #ffffff;
  }

  .pro-locked-notice {
    margin-top: 10px;
    padding: 10px 14px;
    background: linear-gradient(
      135deg,
      rgba(41, 171, 226, 0.12) 0%,
      rgba(0, 0, 0, 0.3) 100%
    );
    border: 1.5px solid rgba(41, 171, 226, 0.4);
    border-radius: 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-self: stretch;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  .pro-notice-title {
    font-size: 15px;
    font-weight: 900;
    color: #ffffff;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    text-shadow: 0 0 8px rgba(41, 171, 226, 0.6);
  }

  .pro-notice-subtitle {
    font-size: 16px;
    color: rgba(255, 255, 255, 0.75);
    margin-bottom: 4px;
  }

  .pro-kofi-banner-link {
    font-family: var(--font-display);
    font-size: 16px;
    font-weight: 900;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #ffffff;
    background: #29abe2;
    border: 1px solid #00b9fe;
    padding: 9px 20px;
    border-radius: 4px;
    text-align: center;
    text-decoration: none;
    transition: all 0.2s ease;
    display: inline-block;
    align-self: flex-start;
    clip-path: polygon(5px 0%, 100% 0%, calc(100% - 5px) 100%, 0% 100%);
  }

  .pro-kofi-banner-link:hover {
    background: #00b9fe;
    box-shadow: 0 0 8px rgba(41, 171, 226, 0.6);
    transform: scale(1.02);
  }

  /* Badges */
  .class-badge-container {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    justify-content: space-between;
    height: 100%;
    align-self: stretch;
    min-height: 58px;
  }

  .class-badge {
    display: flex;
    align-items: center;
    font-family: var(--font-display);
    font-weight: 900;
    border-radius: 4px;
    border: 1.5px solid #000;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    overflow: hidden;
  }

  .badge-letter {
    padding: 2px 6px;
    color: #000;
    font-size: 15px;
  }

  .badge-rating {
    background: #ffffff;
    color: #000000;
    padding: 2px 6px;
    font-size: 15px;
    border-left: 1px solid #000;
  }

  /* Colors matching typical Forza classes */
  .badge-r {
    background: linear-gradient(135deg, #fe0288 0%, #ff007f 100%);
  }
  .badge-r .badge-letter {
    color: #ffffff;
  }
  .badge-r .badge-rating {
    background: #111;
    color: var(--fh6-pink);
    border-left-color: #000;
  }
  .badge-s2 {
    background: #0072ff;
  }
  .badge-s2 .badge-letter {
    color: #ffffff;
  }
  .badge-pro {
    background: linear-gradient(135deg, #29abe2 0%, #00b9fe 100%);
  }
  .badge-pro .badge-letter {
    color: #ffffff;
  }
  .badge-pro .badge-rating {
    background: #111;
    color: #29abe2;
    border-left-color: #000;
  }

  /* Radio Indicator */
  .radio-indicator {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 1.5px solid rgba(255, 255, 255, 0.3);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
    margin-top: auto;
  }

  .car-card.selected .radio-indicator {
    border-color: var(--theme);
  }

  .radio-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: transparent;
    transition:
      background-color 0.2s,
      transform 0.2s;
    transform: scale(0.6);
  }

  .car-card.selected .radio-dot {
    background: var(--theme);
    transform: scale(1);
    box-shadow: 0 0 6px rgba(255, 255, 255, 0.2);
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
    background: var(--theme);
    border: 1.5px solid var(--theme);
    color: #1a2000;
    transition:
      background-color 0.2s,
      border-color 0.2s,
      box-shadow 0.2s;
  }

  .action-btn.save:hover {
    background: #ffffff;
    border-color: #ffffff;
    color: #1a2000;
    box-shadow: 0 0 12px rgba(255, 255, 255, 0.25);
  }
</style>
