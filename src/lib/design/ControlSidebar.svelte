<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { BotConfig } from "$lib/mock";

  export let state = "idle";
  export let stagesEnabled: BotConfig["stages_enabled"] = {
    stage1: true,
    stage2: true,
    stage3: true,
    stage4: true,
  };
  export let loopCount = 0;
  export let creditsBudget = 50000;
  export let stage1Duration = 6;
  export let stage2Iterations = 2;
  export let stage3Iterations = 2;
  export let stage4Iterations = 2;

  const dispatch = createEventDispatcher<{
    cmd: { cmd: string };
    config: {
      stagesEnabled: BotConfig["stages_enabled"];
      loopCount: number;
      creditsBudget: number;
      stage1Duration: number;
      stage2Iterations: number;
      stage3Iterations: number;
      stage4Iterations: number;
    };
  }>();

  $: isIdle = state === "idle";
  $: isRunning = state === "running" || state.startsWith("stage_");
  $: isPaused = state === "paused";

  function emit(cmd: string) {
    dispatch("cmd", { cmd });
  }
  function onStart() {
    emit("start");
  }
  function onStop() {
    emit("stop");
  }
  function onPause() {
    emit("pause");
  }
  function onResume() {
    emit("resume");
  }

  function onConfigChange() {
    if (loopCount === null || loopCount === undefined || loopCount < 0) {
      loopCount = 0;
    } else if (loopCount > 9999) {
      loopCount = 9999;
    }

    if (
      stage1Duration === null ||
      stage1Duration === undefined ||
      stage1Duration < 1
    ) {
      stage1Duration = 1;
    } else if (stage1Duration > 1440) {
      stage1Duration = 1440;
    }

    if (
      stage2Iterations === null ||
      stage2Iterations === undefined ||
      stage2Iterations < 1
    ) {
      stage2Iterations = 1;
    } else if (stage2Iterations > 999) {
      stage2Iterations = 999;
    }

    if (
      stage3Iterations === null ||
      stage3Iterations === undefined ||
      stage3Iterations < 1
    ) {
      stage3Iterations = 1;
    } else if (stage3Iterations > 999) {
      stage3Iterations = 999;
    }

    if (
      stage4Iterations === null ||
      stage4Iterations === undefined ||
      stage4Iterations < 1
    ) {
      stage4Iterations = 1;
    } else if (stage4Iterations > 999) {
      stage4Iterations = 999;
    }

    dispatch("config", {
      stagesEnabled,
      loopCount,
      creditsBudget,
      stage1Duration,
      stage2Iterations,
      stage3Iterations,
      stage4Iterations,
    });
  }

  $: hint = loopCount === 0 ? "∞" : `${loopCount}×`;

  interface StageRow {
    key: "stage1" | "stage2" | "stage3" | "stage4";
    label: string;
    field: string;
    unit: string;
    min: number;
  }

  const STAGE_ROWS: StageRow[] = [
    {
      key: "stage1",
      label: "S1 · Colossus",
      field: "duration",
      unit: "min",
      min: 1,
    },
    {
      key: "stage2",
      label: "S2 · Farm SP",
      field: "stage2Iters",
      unit: "races",
      min: 1,
    },
    {
      key: "stage3",
      label: "S3 · Buy Cars in Journal",
      field: "stage3Iters",
      unit: "buys",
      min: 1,
    },
    {
      key: "stage4",
      label: "S4 · Wheelspin",
      field: "stage4Iters",
      unit: "runs",
      min: 1,
    },
  ];

  const STAGE_COLORS: Record<
    "stage1" | "stage2" | "stage3" | "stage4",
    string
  > = {
    stage1: "#ffcc00",
    stage2: "#2cbca4",
    stage3: "#ef6233",
    stage4: "var(--fh6-lime-dim)",
  };

  $: lapsPerLoop = Math.floor(stage1Duration / 6);
  $: creditsPerLoop = lapsPerLoop * 160000;
  $: totalCredits = creditsPerLoop;

  $: totalSP = stage2Iterations * 10;
  $: totalDurationSec = stage2Iterations * 30;

  $: totalSpentCredits = stage3Iterations * 86000;

  $: totalSPActive = totalSP;
  $: totalDurationSecActive = totalDurationSec;

  $: enabledStagesCount = Object.values(stagesEnabled).filter(Boolean).length;
  $: isSoloStage = enabledStagesCount === 1;
  $: loopsDisabled = !isIdle || enabledStagesCount <= 1;

  $: totalTimePerLoopSec =
    (stagesEnabled.stage1 ? stage1Duration * 60 : 0) +
    (stagesEnabled.stage2 ? stage2Iterations * 30 : 0);
  $: overallTimeSec =
    loopCount === 0 ? totalTimePerLoopSec : totalTimePerLoopSec * loopCount;

  $: totalSpentCreditsActive = totalSpentCredits;
  $: totalBuysActive = stage3Iterations;

  $: stage4SPActive = stage4Iterations * 30;
  $: stage4BuysActive = stage4Iterations;

  $: wSpinsPerLoop = stagesEnabled.stage4 ? stage4Iterations : 0;
  $: overallWSpins = wSpinsPerLoop * loopCount;
  $: wSpinsPerHour =
    totalTimePerLoopSec > 0 ? (wSpinsPerLoop * 3600) / totalTimePerLoopSec : 0;

  let loopsFocused = false;
  let loopsInputValue = "∞";

  $: if (!loopsFocused) {
    loopsInputValue = loopCount === 0 ? "∞" : String(loopCount);
  }

  function handleLoopsFocus() {
    loopsFocused = true;
    if (loopCount === 0) {
      loopsInputValue = "";
    } else {
      loopsInputValue = String(loopCount);
    }
  }

  function handleLoopsBlur() {
    loopsFocused = false;
    let parsed = parseInt(loopsInputValue, 10);
    if (isNaN(parsed) || parsed < 0) {
      parsed = 0;
    } else if (parsed > 9999) {
      parsed = 9999;
    }
    loopCount = parsed;
    onConfigChange();
  }

  function handleLoopsInput() {
    let parsed = parseInt(loopsInputValue, 10);
    if (isNaN(parsed) || parsed < 0) {
      parsed = 0;
    } else if (parsed > 9999) {
      parsed = 9999;
      loopsInputValue = "9999";
    }
    loopCount = parsed;
    onConfigChange();
  }

  $: isDeficit =
    stagesEnabled.stage1 &&
    stagesEnabled.stage3 &&
    totalCredits < totalSpentCreditsActive;

  $: isSPDeficit =
    stagesEnabled.stage2 &&
    stagesEnabled.stage4 &&
    totalSPActive < stage4SPActive;

  $: isCarDeficit =
    stagesEnabled.stage3 &&
    stagesEnabled.stage4 &&
    stage3Iterations < stage4Iterations;

  function handleWheel(
    e: WheelEvent,
    key:
      | "stage1Duration"
      | "stage2Iterations"
      | "stage3Iterations"
      | "stage4Iterations"
      | "loopCount",
  ) {
    if (!isIdle) return;

    e.preventDefault();
    const step = e.deltaY < 0 ? 1 : -1;

    if (key === "stage1Duration" && stagesEnabled.stage1) {
      stage1Duration = Math.min(1440, Math.max(1, stage1Duration + step));
    } else if (key === "stage2Iterations" && stagesEnabled.stage2) {
      stage2Iterations = Math.min(999, Math.max(1, stage2Iterations + step));
    } else if (key === "stage3Iterations" && stagesEnabled.stage3) {
      stage3Iterations = Math.min(999, Math.max(1, stage3Iterations + step));
    } else if (key === "stage4Iterations" && stagesEnabled.stage4) {
      stage4Iterations = Math.min(999, Math.max(1, stage4Iterations + step));
    } else if (key === "loopCount" && !loopsDisabled) {
      loopCount = Math.min(9999, Math.max(0, loopCount + step));
      loopsInputValue = loopsFocused
        ? loopCount === 0
          ? ""
          : String(loopCount)
        : loopCount === 0
          ? "∞"
          : String(loopCount);
    }

    onConfigChange();
  }

  function splitValueAndUnit(str: string): { val: string; unit: string } {
    const parts = str.trim().split(/\s+/);
    if (parts.length >= 2) {
      return { val: parts[0], unit: parts[1] };
    }
    return { val: str, unit: "" };
  }

  function formatCompactValue(val: number, maxLen = 8, sign = ""): string {
    const prefixes = [
      "",
      "k",
      "M",
      "B",
      "T",
      "Qa",
      "Qi",
      "Sx",
      "Sp",
      "Oc",
      "No",
      "Dc",
    ];
    let num = Math.abs(val);
    let unitIdx = 0;
    while (num >= 999.995 && unitIdx < prefixes.length - 1) {
      num /= 1000;
      unitIdx++;
    }
    const prefix = prefixes[unitIdx];

    const maxNumLen = maxLen - sign.length - prefix.length;

    let formatted = "";
    for (let d = 2; d >= 0; d--) {
      formatted = num
        .toFixed(d)
        .replace(/\.00$/, "")
        .replace(/(\.[0-9])0$/, "$1");
      if (formatted.length <= maxNumLen) {
        break;
      }
    }

    if (formatted.length > maxNumLen) {
      formatted = num.toExponential(1).replace(/\.0e/, "e").replace(/\+/, "");
      if (formatted.length > maxNumLen) {
        formatted = num.toExponential(0).replace(/\+/, "");
      }
      if (formatted.length > maxNumLen) {
        formatted = formatted.slice(0, maxNumLen);
      }
    }

    return `${sign}${formatted}${prefix}`;
  }

  function formatCredits(val: number): string {
    if (val === 0) return "0 Cr";
    const isNegative = val < 0;
    const prefixSign = isNegative ? "-" : "~";
    return `${formatCompactValue(val, 8, prefixSign)} Cr`;
  }

  function formatLaps(val: number): string {
    if (val === 0) return "0 laps";
    const label = val === 1 ? "lap" : "laps";
    return `${formatCompactValue(val, 8)} ${label}`;
  }

  function formatDuration(sec: number): string {
    const h = Math.floor(sec / 3600);
    const m = Math.floor((sec % 3600) / 60);
    const s = sec % 60;

    if (h > 0) {
      if (h >= 1000) {
        return `${formatCompactValue(h, 6)}h`;
      }
      return m > 0 ? `${h}h ${m}m` : `${h}h`;
    }
    if (m > 0) {
      return s > 0 ? `${m}m ${s}s` : `${m}m`;
    }
    return `${s}s`;
  }
</script>

<aside class="sidebar">
  <!-- ── Left: CONTROL header + action buttons ──────────────── -->
  <div class="ctrl-left">
    <div class="ctrl-header">
      <span class="ctrl-title">CONTROL</span>
    </div>
    <div class="btn-group">
      <!-- START / PAUSE / RESUME -->
      {#if isIdle || (!isRunning && !isPaused)}
        <button class="action-btn start" on:click={onStart}>
          <svg class="btn-ico" viewBox="0 0 12 12" fill="currentColor"
            ><polygon points="2,1 11,6 2,11" /></svg
          >
          START
        </button>
      {:else if isRunning}
        <button class="action-btn pause" on:click={onPause}>
          <svg class="btn-ico" viewBox="0 0 12 12" fill="currentColor"
            ><rect x="2" y="1" width="3" height="10" /><rect
              x="7"
              y="1"
              width="3"
              height="10"
            /></svg
          >
          PAUSE
        </button>
      {:else if isPaused}
        <button class="action-btn resume" on:click={onResume}>
          <svg class="btn-ico" viewBox="0 0 12 12" fill="currentColor"
            ><polygon points="2,1 11,6 2,11" /></svg
          >
          RESUME
        </button>
      {/if}

      <!-- STOP — always visible, disabled when idle -->
      <button class="action-btn stop" on:click={onStop} disabled={isIdle}>
        <svg class="btn-ico" viewBox="0 0 12 12" fill="currentColor"
          ><rect x="1" y="1" width="10" height="10" /></svg
        >
        STOP
      </button>
    </div>
  </div>

  <!-- ── Vertical divider ──────────────────────────────────── -->
  <div class="v-sep"></div>

  <!-- ── Right: settings ───────────────────────────────────── -->
  <div class="ctrl-right">
    <!-- Stage config -->
    <!-- Settings 1: Stage config -->
    <div class="cfg-section">
      <div class="cfg-header">
        <span class="cfg-title">SETTINGS</span>
      </div>
      <div class="cfg-body">
        {#each STAGE_ROWS as row}
          {@const en = stagesEnabled[row.key]}
          <div class="cfg-row" class:faded={!en}>
            <div
              class="cfg-dot"
              style="background: {en
                ? STAGE_COLORS[row.key]
                : 'var(--text-dim)'}"
            ></div>
            <span class="cfg-label">{row.label}</span>
            <div class="cfg-input-wrap">
              {#if row.field === "duration"}
                <input
                  class="cfg-input"
                  type="number"
                  min={row.min}
                  step="1"
                  bind:value={stage1Duration}
                  on:change={onConfigChange}
                  on:wheel|nonpassive={(e) => handleWheel(e, "stage1Duration")}
                  disabled={!isIdle || !en}
                />
              {:else if row.field === "stage2Iters"}
                <input
                  class="cfg-input"
                  type="number"
                  min={row.min}
                  step="1"
                  bind:value={stage2Iterations}
                  on:change={onConfigChange}
                  on:wheel|nonpassive={(e) =>
                    handleWheel(e, "stage2Iterations")}
                  disabled={!isIdle || !en}
                />
              {:else if row.field === "stage3Iters"}
                <input
                  class="cfg-input"
                  type="number"
                  min={row.min}
                  step="1"
                  bind:value={stage3Iterations}
                  on:change={onConfigChange}
                  on:wheel|nonpassive={(e) =>
                    handleWheel(e, "stage3Iterations")}
                  disabled={!isIdle || !en}
                />
              {:else if row.field === "stage4Iters"}
                <input
                  class="cfg-input"
                  type="number"
                  min={row.min}
                  step="1"
                  bind:value={stage4Iterations}
                  on:change={onConfigChange}
                  on:wheel|nonpassive={(e) =>
                    handleWheel(e, "stage4Iterations")}
                  disabled={!isIdle || !en}
                />
              {/if}
            </div>
            <span class="cfg-unit">{row.unit}</span>
          </div>
        {/each}
      </div>
    </div>

    <div class="v-sep"></div>

    <!-- Settings 2: Global config -->
    <div class="cfg-section">
      <div class="cfg-header">
        <span class="cfg-title">ESTIMATION</span>
      </div>
      <div class="cfg-body estimation-body">
        <!-- Calculations at the top -->
        <div class="calc-panel-compact">
          {#if enabledStagesCount > 0}
            <!-- Stage 1 Row -->
            {#if stagesEnabled.stage1}
              {@const parts = splitValueAndUnit(formatCredits(totalCredits))}
              <div
                class="calc-yield-value"
                class:highlight-yellow={!isDeficit}
                class:highlight-pink={isDeficit}
              >
                <span class="calc-val-num">{parts.val}</span>
                <span class="calc-val-unit">{parts.unit}</span>
                <span
                  class="calc-laps-inline"
                  class:text-warn={lapsPerLoop === 0}
                >
                  {formatLaps(lapsPerLoop)}
                </span>
              </div>
            {/if}

            <!-- Stage 2 Row -->
            {#if stagesEnabled.stage2}
              <div
                class="calc-yield-value"
                class:highlight-teal={!isSPDeficit && totalSPActive <= 999}
                class:highlight-pink={isSPDeficit || totalSPActive > 999}
              >
                <span class="calc-val-num">+{Math.min(totalSPActive, 999)}</span
                >
                <span class="calc-val-unit">SP</span>
                <span class="calc-laps-inline">
                  ~{formatDuration(totalDurationSecActive)}
                </span>
              </div>
            {/if}

            <!-- Stage 3 Row -->
            {#if stagesEnabled.stage3}
              {@const parts = splitValueAndUnit(
                "-" + formatCredits(totalSpentCreditsActive).replace("~", ""),
              )}
              <div
                class="calc-yield-value"
                class:highlight-orange={!isDeficit && !isCarDeficit}
                class:highlight-pink={isDeficit || isCarDeficit}
              >
                <span class="calc-val-num">{parts.val}</span>
                <span class="calc-val-unit">{parts.unit}</span>
                <span class="calc-laps-inline">
                  {formatCompactValue(totalBuysActive, 8)}
                  {totalBuysActive === 1 ? "buy" : "buys"}
                </span>
              </div>
            {/if}

            <!-- Stage 4 Row -->
            {#if stagesEnabled.stage4}
              <div
                class="calc-yield-value"
                class:highlight-green={!isSPDeficit && !isCarDeficit}
                class:highlight-pink={isSPDeficit || isCarDeficit}
              >
                <span class="calc-val-num"
                  >{formatCompactValue(stage4SPActive, 8, "-")}</span
                >
                <span class="calc-val-unit">SP</span>
                <span class="calc-laps-inline">
                  {formatCompactValue(stage4BuysActive, 8)}
                  {stage4BuysActive === 1 ? "W.Spin" : "W.Spins"}
                </span>
              </div>
            {/if}
          {:else}
            <div class="calc-placeholder-compact">
              ℹ Select Stage 1, 2, 3 or combinations to calculate yield
            </div>
          {/if}
        </div>

        <!-- Loops input at the bottom -->
        <div class="cfg-row loops-row" class:faded={loopsDisabled}>
          <div
            class="cfg-dot"
            style="background: {loopsDisabled
              ? 'var(--text-dim)'
              : 'var(--fh6-teal)'}"
          ></div>
          <span class="cfg-label loops-label">Loops</span>
          <div class="cfg-input-wrap">
            <input
              class="cfg-input"
              class:infinity-font={loopsInputValue === "∞"}
              type="text"
              inputmode="numeric"
              pattern="[0-9]*"
              bind:value={loopsInputValue}
              on:focus={handleLoopsFocus}
              on:blur={handleLoopsBlur}
              on:input={handleLoopsInput}
              on:wheel|nonpassive={(e) => handleWheel(e, "loopCount")}
              disabled={loopsDisabled}
              title="0 = Infinite"
            />
          </div>

          <!-- Estimated total time displayed on the right -->
          {#if enabledStagesCount > 0}
            <div class="loops-total-time">
              <span class="total-time-line">
                {#if loopCount === 0}
                  ~{formatDuration(totalTimePerLoopSec)}/loop
                {:else}
                  ~{formatDuration(overallTimeSec)}
                {/if}
              </span>
              <span class="total-wspins-line">
                {#if loopCount === 0}
                  ~{formatCompactValue(wSpinsPerHour, 8)} W.Spins/h
                {:else}
                  {formatCompactValue(overallWSpins, 8)}
                  {overallWSpins === 1 ? "W.Spin" : "W.Spins"}
                {/if}
              </span>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</aside>

<style>
  /* ── Shell ─────────────────────────────────────────────────── */
  .sidebar {
    display: flex;
    flex-direction: row;
    align-items: stretch;
    overflow: hidden; /* no scroll */
    background: linear-gradient(160deg, #1b5258 0%, #267060 100%);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.18);
    width: 100%;
    height: 171px;
  }

  /* ── Left block ────────────────────────────────────────────── */
  .ctrl-left {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    flex-shrink: 0;
    min-width: 180px;
    background: rgba(0, 0, 0, 0.18);
    overflow: hidden;
  }

  /* Header: CONTROL label with lime bottom border */
  .ctrl-header {
    padding: 0 20px;
    height: 36px;
    border-bottom: 2px solid var(--fh6-lime);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .ctrl-title {
    font-size: 13px;
    font-weight: 900;
    letter-spacing: 0.22em;
    color: #ffffff;
    text-transform: uppercase;
    white-space: nowrap;
  }

  /* ── Action buttons ────────────────────────────────────────── */
  .btn-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: stretch;
    padding: 12px 20px;
    flex: 1;
    justify-content: center;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 11px 20px;
    font-size: 13px;
    font-weight: 900;
    letter-spacing: 0.16em;
    border: 2px solid;
    background: transparent;
    clip-path: var(--clip-btn);
    transition:
      box-shadow 0.2s,
      background 0.2s,
      transform 0.1s;
    cursor: pointer;
    position: relative;
    overflow: hidden;
    white-space: nowrap;
    flex-shrink: 0;
  }

  /* Shimmer */
  .action-btn::after {
    content: "";
    position: absolute;
    top: 0;
    left: -100%;
    width: 60%;
    height: 100%;
    background: linear-gradient(
      90deg,
      transparent,
      rgba(255, 255, 255, 0.3),
      transparent
    );
    transform: skewX(-20deg);
    transition: left 0.4s ease;
    pointer-events: none;
  }
  .action-btn:hover::after {
    left: 150%;
  }
  .action-btn:active {
    transform: scale(0.97);
  }
  .action-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
    border-color: rgba(255, 255, 255, 0.2) !important;
    background: rgba(255, 255, 255, 0.06) !important;
    color: rgba(255, 255, 255, 0.3) !important;
    box-shadow: none !important;
  }
  .action-btn:disabled::after {
    display: none;
  }

  .start,
  .resume {
    border-color: var(--fh6-lime);
    background: var(--fh6-lime);
    color: #1a2000;
  }
  .start:hover,
  .resume:hover {
    background: var(--fh6-lime-dim);
    border-color: var(--fh6-lime-dim);
    box-shadow: 0 4px 16px var(--fh6-lime-glow);
  }

  .stop {
    border-color: var(--fh6-pink);
    color: var(--fh6-pink);
  }
  .stop:hover {
    background: var(--fh6-pink-faint);
    box-shadow: 0 4px 16px var(--fh6-pink-glow);
  }

  .pause {
    border-color: var(--fh6-teal);
    color: var(--fh6-teal);
  }
  .pause:hover {
    background: var(--fh6-teal-faint);
    box-shadow: 0 4px 16px var(--fh6-teal-glow);
  }

  .btn-ico {
    width: 9px;
    height: 9px;
    flex-shrink: 0;
  }

  /* ── Vertical divider ────────────────────────────────────────── */
  .v-sep {
    width: 1px;
    background: rgba(255, 255, 255, 0.15);
    flex-shrink: 0;
    align-self: stretch;
  }

  /* ── Right block ───────────────────────────────────────────── */
  .ctrl-right {
    display: flex;
    flex-direction: row;
    align-items: stretch;
    flex: 1;
    overflow: hidden;
    min-width: 0;
  }

  /* ── Config section ────────────────────────────────────────── */
  .cfg-section {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    flex: 1;
    min-width: 0;
    overflow: hidden;
    background: rgba(0, 0, 0, 0.1);
  }

  /* Header styled like ctrl-header */
  .cfg-header {
    padding: 0 16px;
    height: 36px;
    border-bottom: 2px solid var(--fh6-lime);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.15);
  }

  .cfg-title {
    font-size: 13px;
    font-weight: 900;
    letter-spacing: 0.22em;
    color: #ffffff;
    text-transform: uppercase;
    white-space: nowrap;
  }

  /* Content area */
  .cfg-body {
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 10px 14px;
    justify-content: center;
    flex: 1;
  }

  .cfg-row {
    display: flex;
    align-items: center;
    gap: 8px;
    transition: opacity 0.2s;
    min-height: 22px;
    white-space: nowrap;
  }

  .cfg-row.faded {
    opacity: 0.28;
  }

  .cfg-dot {
    width: 5px;
    height: 16px;
    border-radius: 2px;
    flex-shrink: 0;
  }

  .cfg-label {
    font-size: 12px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.85);
    white-space: nowrap;
    letter-spacing: 0.03em;
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .cfg-input-wrap {
    flex-shrink: 0;
    width: 48px;
  }

  .cfg-input {
    width: 100%;
    height: 23px;
    padding: 0 3px;
    text-align: center;
    font-family: var(--font-mono);
    font-size: 13px;
    font-weight: 600;
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid rgba(255, 255, 255, 0.18);
    color: #ffffff;
    border-radius: var(--radius-sm);
    transition:
      border-color 0.15s,
      box-shadow 0.15s;
  }

  .cfg-input:focus {
    border-color: var(--fh6-lime-dim);
    box-shadow: 0 0 0 2px var(--fh6-lime-faint);
    outline: none;
  }

  .cfg-input:disabled {
    opacity: 0.35;
  }

  .cfg-input.infinity-font {
    font-size: 17px;
    font-weight: 700;
  }

  /* Hide standard numeric spin buttons */
  .cfg-input::-webkit-outer-spin-button,
  .cfg-input::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
  .cfg-input[type="number"] {
    -moz-appearance: textfield;
    appearance: textfield;
  }

  .cfg-unit {
    font-size: 11px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.4);
    flex-shrink: 0;
    white-space: nowrap;
    letter-spacing: 0.05em;
    width: 32px;
  }

  .estimation-body {
    justify-content: flex-start !important;
    gap: 5px;
  }

  .calc-panel-compact {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-height: 0;
  }

  .calc-yield-value {
    display: grid;
    grid-template-columns: 80px 25px 1fr;
    align-items: baseline;
    font-family: var(--font-mono);
    font-size: 16px;
    font-weight: 900;
    letter-spacing: 0.02em;
    line-height: 1.2;
    white-space: nowrap;
  }

  .calc-val-num {
    text-align: left;
  }

  .calc-val-unit {
    font-size: 14px;
  }

  .calc-yield-value.highlight-yellow {
    color: #ffcc00;
    text-shadow: 0 0 10px rgba(255, 204, 0, 0.22);
  }

  .calc-yield-value.highlight-green {
    color: var(--fh6-lime-dim);
    text-shadow: 0 0 10px rgba(168, 212, 0, 0.22);
  }

  .calc-yield-value.highlight-teal {
    color: var(--fh6-teal);
    text-shadow: 0 0 10px rgba(44, 188, 164, 0.22);
  }

  .calc-yield-value.highlight-pink {
    color: var(--fh6-pink);
    text-shadow: 0 0 10px rgba(254, 2, 136, 0.22);
  }

  .calc-yield-value.highlight-orange {
    color: #ef6233;
    text-shadow: 0 0 10px rgba(239, 98, 51, 0.22);
  }

  .calc-placeholder-compact {
    font-size: 10px;
    line-height: 1.3;
    color: rgba(255, 255, 255, 0.38);
    font-weight: 500;
  }

  .calc-laps-inline {
    font-size: 11px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.4);
    letter-spacing: 0.02em;
    margin-left: 0;
    text-shadow: none !important;
    white-space: nowrap;
  }

  .calc-laps-inline.text-warn {
    color: var(--fh6-pink) !important;
    text-shadow: 0 0 8px rgba(254, 2, 136, 0.22) !important;
  }

  .loops-row {
    margin-top: auto;
  }

  .loops-label {
    flex: none !important;
    width: auto !important;
    margin-right: 4px;
  }

  .loops-total-time {
    margin-left: auto;
    margin-right: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    font-size: 13px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.5);
    font-family: var(--font-display);
    letter-spacing: 0.02em;
    line-height: 1.2;
    text-align: center;
  }

  .total-time-line {
    color: rgba(255, 255, 255, 0.85);
  }

  .total-wspins-line {
    color: var(--fh6-lime-dim);
    font-size: 11px;
    opacity: 0.9;
  }
</style>
